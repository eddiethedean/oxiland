#[cfg(test)]
use std::cell::Cell;
use std::path::Path;

use fjall::{Config, Keyspace, Partition, PartitionCreateOptions, PersistMode};
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::Quad;
use oxigraph::store::Store;

use crate::{Error, Result};

const QUADS_PARTITION: &str = "oxiland_quads";
pub(crate) const META_KEY: &str = "__oxiland/meta";
pub(crate) const FORMAT_VERSION: u32 = 1;
pub(crate) const FORMAT_OXILAND: &str = "0.4.0";

#[cfg(test)]
thread_local! {
    static DISK_INSERT_FAULT: Cell<bool> = const { Cell::new(false) };
    /// When set, fail after inserting desired keys but before removing orphans.
    static DISK_REPLACE_FAULT: Cell<bool> = const { Cell::new(false) };
    /// When set, fail after orphan removal but before persist.
    static DISK_REPLACE_PERSIST_FAULT: Cell<bool> = const { Cell::new(false) };
}

/// Durable quad storage backed by [Fjall](https://github.com/fjall-rs/fjall).
///
/// Oxigraph still provides the in-memory query engine; Fjall holds the durable
/// copy of every quad under Oxiland format v1 (ADR-006).
#[derive(Clone)]
pub(crate) struct DiskStore {
    keyspace: Keyspace,
    quads: Partition,
}

impl DiskStore {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn open(path: &Path) -> Result<Self> {
        Self::open_with_create(path, true)
    }

    pub(crate) fn open_with_create(path: &Path, create: bool) -> Result<Self> {
        if create {
            std::fs::create_dir_all(path).map_err(|error| Error::OpenStore {
                path: path.to_owned(),
                message: error.to_string(),
            })?;
        } else if !path.exists() {
            return Err(Error::OpenStore {
                path: path.to_owned(),
                message: "path does not exist and OpenOptions::create(false)".into(),
            });
        }

        let keyspace = Config::new(path).open().map_err(|error| Error::OpenStore {
            path: path.to_owned(),
            message: error.to_string(),
        })?;
        let quads = keyspace
            .open_partition(QUADS_PARTITION, PartitionCreateOptions::default())
            .map_err(|error| Error::OpenStore {
                path: path.to_owned(),
                message: error.to_string(),
            })?;

        Ok(Self { keyspace, quads })
    }

    pub(crate) fn ensure_format_v1(&self, path: &Path) -> Result<()> {
        match self.read_format_version()? {
            Some(version) if version == FORMAT_VERSION => Ok(()),
            Some(version) => Err(Error::Unsupported(format!(
                "Oxiland on-disk format version {version} is not supported by this build (expected {FORMAT_VERSION})"
            ))),
            None => {
                if self.has_quad_keys()? {
                    Err(Error::Unsupported(format!(
                        "store at {} looks like a pre-0.4 experimental Oxiland directory; call Model::migrate_legacy_store before opening",
                        path.display()
                    )))
                } else {
                    self.write_format_v1_meta()
                }
            }
        }
    }

    pub(crate) fn migrate_legacy_to_v1(&self) -> Result<()> {
        if self.read_format_version()?.is_some() {
            return Ok(());
        }
        // Validate every non-meta key parses as a quad.
        for entry in self.quads.iter() {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            let key = std::str::from_utf8(&key).map_err(|error| {
                Error::Storage(format!("persisted quad key was not UTF-8: {error}"))
            })?;
            if key == META_KEY {
                continue;
            }
            let _ = parse_quad(key)?;
        }
        self.write_format_v1_meta()
    }

    pub(crate) fn write_format_v1_meta(&self) -> Result<()> {
        let meta =
            format!("{{\"format_version\":{FORMAT_VERSION},\"oxiland\":\"{FORMAT_OXILAND}\"}}");
        self.quads
            .insert(META_KEY.as_bytes(), meta.as_bytes())
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.keyspace
            .persist(PersistMode::SyncAll)
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn read_format_version(&self) -> Result<Option<u32>> {
        match self
            .quads
            .get(META_KEY.as_bytes())
            .map_err(|error| Error::Storage(error.to_string()))?
        {
            None => Ok(None),
            Some(bytes) => {
                let text = std::str::from_utf8(&bytes).map_err(|error| {
                    Error::Storage(format!("format metadata was not UTF-8: {error}"))
                })?;
                let version = parse_format_version(text)?;
                Ok(Some(version))
            }
        }
    }

    fn has_quad_keys(&self) -> Result<bool> {
        for entry in self.quads.iter() {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            let key = std::str::from_utf8(&key).map_err(|error| {
                Error::Storage(format!("persisted quad key was not UTF-8: {error}"))
            })?;
            if key != META_KEY {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(crate) fn load_into(&self, store: &Store) -> Result<()> {
        for entry in self.quads.iter() {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            let key = std::str::from_utf8(&key).map_err(|error| {
                Error::Storage(format!("persisted quad key was not UTF-8: {error}"))
            })?;
            if key == META_KEY {
                continue;
            }
            let quad = parse_quad(key)?;
            store
                .insert(&quad)
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        Ok(())
    }

    pub(crate) fn sync(&self) -> Result<()> {
        self.keyspace
            .persist(PersistMode::SyncAll)
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn clear_quads(&self) -> Result<()> {
        let mut keys = Vec::new();
        for entry in self.quads.iter() {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            let key = std::str::from_utf8(&key).map_err(|error| {
                Error::Storage(format!("persisted quad key was not UTF-8: {error}"))
            })?;
            if key != META_KEY {
                keys.push(key.to_owned());
            }
        }
        for key in &keys {
            self.quads
                .remove(key.as_bytes())
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        self.sync()
    }

    pub(crate) fn insert(&self, quad: &Quad) -> Result<()> {
        #[cfg(test)]
        if DISK_INSERT_FAULT.with(Cell::get) {
            return Err(Error::Storage("injected disk insert fault".into()));
        }
        let key = quad_key(quad);
        self.quads
            .insert(key.as_bytes(), [])
            .map_err(|error| Error::Storage(error.to_string()))?;
        if let Err(error) = self.keyspace.persist(PersistMode::SyncAll) {
            let _ = self.quads.remove(key.as_bytes());
            if let Err(compensate_err) = self.keyspace.persist(PersistMode::SyncAll) {
                return Err(Error::Storage(format!(
                    "durable insert sync failed ({error}); compensation persist also failed ({compensate_err})"
                )));
            }
            return Err(Error::Storage(error.to_string()));
        }
        Ok(())
    }

    /// Removes every durable key whose parsed quad is RDF-equal to `quad`.
    ///
    /// Scanning by RDF equality (not opaque key string) cleans alternate lexical
    /// forms left by older builds and matches Oxigraph term equality.
    pub(crate) fn remove_rdf_equal(&self, quad: &Quad) -> Result<()> {
        let mut keys = Vec::new();
        for entry in self.quads.iter() {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            let key = std::str::from_utf8(&key).map_err(|error| {
                Error::Storage(format!("persisted quad key was not UTF-8: {error}"))
            })?;
            if key == META_KEY {
                continue;
            }
            let parsed = parse_quad(key)?;
            if quads_rdf_equal(&parsed, quad)? {
                keys.push(key.to_owned());
            }
        }
        for key in &keys {
            self.quads
                .remove(key.as_bytes())
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        if let Err(error) = self.keyspace.persist(PersistMode::SyncAll) {
            for key in &keys {
                let _ = self.quads.insert(key.as_bytes(), []);
            }
            if let Err(compensate_err) = self.keyspace.persist(PersistMode::SyncAll) {
                return Err(Error::Storage(format!(
                    "durable remove sync failed ({error}); compensation persist also failed ({compensate_err})"
                )));
            }
            return Err(Error::Storage(error.to_string()));
        }
        Ok(())
    }

    /// Rewrites durable keys to match `store` after SPARQL Update (0.3).
    ///
    /// Applies inserts then deletes with compensation so a mid-sync failure
    /// leaves the on-disk key set unchanged (pre-update snapshot).
    pub(crate) fn replace_all_from_store(&self, store: &Store) -> Result<()> {
        use std::collections::HashSet;

        let mut desired = HashSet::new();
        for item in store.iter() {
            let quad = item.map_err(|error| Error::Storage(error.to_string()))?;
            desired.insert(quad_key(&quad));
        }

        let mut current = HashSet::new();
        for entry in self.quads.iter() {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            let key = std::str::from_utf8(&key).map_err(|error| {
                Error::Storage(format!("persisted quad key was not UTF-8: {error}"))
            })?;
            if key == META_KEY {
                continue;
            }
            current.insert(key.to_owned());
        }

        let to_insert: Vec<_> = desired.difference(&current).cloned().collect();
        let to_delete: Vec<_> = current.difference(&desired).cloned().collect();

        let mut inserted = Vec::new();
        for key in &to_insert {
            if let Err(error) = self.quads.insert(key.as_bytes(), []) {
                let _ = self.compensate_replace(&inserted, &[]);
                return Err(Error::Storage(error.to_string()));
            }
            inserted.push(key.clone());
        }

        #[cfg(test)]
        if DISK_REPLACE_FAULT.with(Cell::get) {
            self.compensate_replace(&inserted, &[])?;
            return Err(Error::Storage(
                "injected disk replace fault after inserts".into(),
            ));
        }

        let mut deleted = Vec::new();
        for key in &to_delete {
            if let Err(error) = self.quads.remove(key.as_bytes()) {
                let _ = self.compensate_replace(&inserted, &deleted);
                return Err(Error::Storage(error.to_string()));
            }
            deleted.push(key.clone());
        }

        #[cfg(test)]
        if DISK_REPLACE_PERSIST_FAULT.with(Cell::get) {
            self.compensate_replace(&inserted, &deleted)?;
            return Err(Error::Storage(
                "injected disk replace fault before persist".into(),
            ));
        }

        if let Err(error) = self.keyspace.persist(PersistMode::SyncAll) {
            self.compensate_replace(&inserted, &deleted)?;
            self.keyspace
                .persist(PersistMode::SyncAll)
                .map_err(|compensate_err| {
                    Error::Storage(format!(
                        "durable replace sync failed ({error}); compensation persist also failed ({compensate_err})"
                    ))
                })?;
            return Err(Error::Storage(error.to_string()));
        }
        Ok(())
    }

    fn compensate_replace(&self, inserted: &[String], deleted: &[String]) -> Result<()> {
        for key in deleted {
            self.quads.insert(key.as_bytes(), []).map_err(|error| {
                Error::Storage(format!("replace compensation insert failed: {error}"))
            })?;
        }
        for key in inserted {
            self.quads.remove(key.as_bytes()).map_err(|error| {
                Error::Storage(format!("replace compensation remove failed: {error}"))
            })?;
        }
        Ok(())
    }
}

fn quad_key(quad: &Quad) -> String {
    format!("{quad} .")
}

fn parse_format_version(meta: &str) -> Result<u32> {
    // Minimal JSON parse: look for "format_version": <int>
    let key = "\"format_version\"";
    let Some(pos) = meta.find(key) else {
        return Err(Error::Storage(
            "format metadata missing format_version".into(),
        ));
    };
    let rest = &meta[pos + key.len()..];
    let rest = rest.trim_start().trim_start_matches(':').trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits
        .parse::<u32>()
        .map_err(|_| Error::Storage("format metadata has invalid format_version".into()))
}

fn parse_quad(key: &str) -> Result<Quad> {
    let mut parsed = RdfParser::from_format(RdfFormat::NQuads).for_reader(key.as_bytes());
    let quad = parsed
        .next()
        .ok_or_else(|| Error::Storage("persisted quad key was empty".into()))?
        .map_err(|error| Error::Storage(error.to_string()))?;
    if parsed.next().is_some() {
        return Err(Error::Storage(
            "persisted quad key contained multiple quads".into(),
        ));
    }
    Ok(quad)
}

/// RDF term equality as used by Oxigraph stores (value-equal typed literals).
fn quads_rdf_equal(left: &Quad, right: &Quad) -> Result<bool> {
    let probe = Store::new().map_err(|error| Error::Storage(error.to_string()))?;
    probe
        .insert(left)
        .map_err(|error| Error::Storage(error.to_string()))?;
    probe
        .contains(right.as_ref())
        .map_err(|error| Error::Storage(error.to_string()))
}

/// Returns the store's canonical quad matching `quad` under RDF equality.
pub(crate) fn stored_matching_quad(store: &Store, quad: &Quad) -> Result<Quad> {
    store
        .quads_for_pattern(
            Some(quad.subject.as_ref()),
            Some(quad.predicate.as_ref()),
            Some(quad.object.as_ref()),
            Some(quad.graph_name.as_ref()),
        )
        .next()
        .ok_or_else(|| {
            Error::Storage("matching quad missing from store after contains check".into())
        })?
        .map_err(|error| Error::Storage(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Model;
    use crate::terms::{self, Literal, Triple};
    use oxigraph::model::{NamedNode, Quad};

    #[test]
    fn duplicate_insert_skips_disk_and_preserves_existing_quad() {
        let dir = tempfile::tempdir().unwrap();
        let model = Model::open(dir.path()).unwrap();
        let statement = Triple::new(
            terms::named_node("https://example.com/s").unwrap(),
            terms::named_node("https://example.com/p").unwrap(),
            Literal::new_simple_literal("x"),
        );
        let quad = Quad::new(
            statement.subject.clone(),
            statement.predicate.clone(),
            statement.object.clone(),
            oxigraph::model::GraphName::DefaultGraph,
        );
        assert!(model.insert_quad(quad.clone()).unwrap());
        assert_eq!(model.len().unwrap(), 1);

        // RDF-equal duplicates must not touch disk (fault would otherwise fire).
        DISK_INSERT_FAULT.with(|flag| flag.set(true));
        assert!(!model.insert_quad(quad).unwrap());
        DISK_INSERT_FAULT.with(|flag| flag.set(false));
        assert_eq!(model.len().unwrap(), 1);
        assert!(model.contains(statement.as_ref()).unwrap());
    }

    #[test]
    fn new_insert_disk_fault_rolls_back_to_disk_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let model = Model::open(dir.path()).unwrap();
        let keep = Triple::new(
            terms::named_node("https://example.com/s").unwrap(),
            terms::named_node("https://example.com/p").unwrap(),
            Literal::new_simple_literal("keep"),
        );
        assert!(model.add(keep.clone()).unwrap());

        let fresh = Triple::new(
            terms::named_node("https://example.com/s2").unwrap(),
            terms::named_node("https://example.com/p").unwrap(),
            Literal::new_simple_literal("new"),
        );
        let quad = Quad::new(
            fresh.subject.clone(),
            fresh.predicate.clone(),
            fresh.object.clone(),
            oxigraph::model::GraphName::DefaultGraph,
        );
        DISK_INSERT_FAULT.with(|flag| flag.set(true));
        let err = model.insert_quad(quad).unwrap_err();
        DISK_INSERT_FAULT.with(|flag| flag.set(false));
        assert!(matches!(err, Error::Storage(_)));
        assert_eq!(model.len().unwrap(), 1);
        assert!(model.contains(keep.as_ref()).unwrap());
        assert!(!model.contains(fresh.as_ref()).unwrap());
    }

    #[test]
    fn sparql_update_replace_fault_rolls_back_memory() {
        use crate::Update;

        let dir = tempfile::tempdir().unwrap();
        let model = Model::open(dir.path()).unwrap();
        let statement = Triple::new(
            terms::named_node("https://example.com/s").unwrap(),
            terms::named_node("https://example.com/p").unwrap(),
            Literal::new_simple_literal("keep"),
        );
        model.add(statement.clone()).unwrap();
        assert_eq!(model.len().unwrap(), 1);

        DISK_REPLACE_FAULT.with(|flag| flag.set(true));
        let err = Update::new(
            "DELETE DATA { <https://example.com/s> <https://example.com/p> \"keep\" } ; INSERT DATA { <https://example.com/s> <https://example.com/p> \"new\" }",
        )
        .execute(&model)
        .unwrap_err();
        DISK_REPLACE_FAULT.with(|flag| flag.set(false));

        assert!(matches!(err, Error::Storage(_)));
        assert_eq!(model.len().unwrap(), 1);
        assert!(model.contains(statement.as_ref()).unwrap());
        assert!(
            !model
                .contains(
                    Triple::new(
                        terms::named_node("https://example.com/s").unwrap(),
                        terms::named_node("https://example.com/p").unwrap(),
                        Literal::new_simple_literal("new"),
                    )
                    .as_ref()
                )
                .unwrap()
        );

        drop(model);
        let reopened = Model::open(dir.path()).unwrap();
        assert_eq!(reopened.len().unwrap(), 1);
        assert!(reopened.contains(statement.as_ref()).unwrap());
    }

    #[test]
    fn sparql_update_persist_fault_keeps_pre_update_disk() {
        use crate::Update;

        let dir = tempfile::tempdir().unwrap();
        let model = Model::open(dir.path()).unwrap();
        let keep = Triple::new(
            terms::named_node("https://example.com/s").unwrap(),
            terms::named_node("https://example.com/p").unwrap(),
            Literal::new_simple_literal("keep"),
        );
        model.add(keep.clone()).unwrap();

        DISK_REPLACE_PERSIST_FAULT.with(|flag| flag.set(true));
        let err = Update::new("DELETE { ?s ?p ?o } INSERT { ?s ?p \"new\" } WHERE { ?s ?p ?o }")
            .execute(&model)
            .unwrap_err();
        DISK_REPLACE_PERSIST_FAULT.with(|flag| flag.set(false));

        assert!(matches!(err, Error::Storage(_)));
        assert_eq!(model.len().unwrap(), 1);
        assert!(model.contains(keep.as_ref()).unwrap());

        drop(model);
        let reopened = Model::open(dir.path()).unwrap();
        assert_eq!(reopened.len().unwrap(), 1);
        assert!(reopened.contains(keep.as_ref()).unwrap());
    }

    #[test]
    fn typed_literal_canonical_remove_must_not_resurrect() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store");
        let integer = NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#integer");
        {
            let model = Model::open(&path).unwrap();
            let statement = Triple::new(
                terms::named_node("https://example.com/s").unwrap(),
                terms::named_node("https://example.com/p").unwrap(),
                Literal::new_typed_literal("01", integer.clone()),
            );
            assert!(model.add(statement).unwrap());
            let canonical = Triple::new(
                terms::named_node("https://example.com/s").unwrap(),
                terms::named_node("https://example.com/p").unwrap(),
                Literal::new_typed_literal("1", integer),
            );
            assert!(model.remove(canonical).unwrap());
            assert_eq!(model.len().unwrap(), 0);
        }
        let reopened = Model::open(&path).unwrap();
        assert_eq!(reopened.len().unwrap(), 0);
    }

    #[test]
    fn duplicate_lexical_forms_must_not_double_persist() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store");
        let integer = NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#integer");
        let model = Model::open(&path).unwrap();
        let s = terms::named_node("https://example.com/s").unwrap();
        let p = terms::named_node("https://example.com/p").unwrap();
        assert!(
            model
                .add(Triple::new(
                    s.clone(),
                    p.clone(),
                    Literal::new_typed_literal("01", integer.clone()),
                ))
                .unwrap()
        );
        assert!(
            !model
                .add(Triple::new(
                    s.clone(),
                    p.clone(),
                    Literal::new_typed_literal("1", integer.clone()),
                ))
                .unwrap()
        );
        assert_eq!(model.len().unwrap(), 1);
        assert!(
            model
                .remove(Triple::new(s, p, Literal::new_typed_literal("1", integer),))
                .unwrap()
        );
        assert_eq!(model.len().unwrap(), 0);
        drop(model);
        let reopened = Model::open(&path).unwrap();
        assert_eq!(reopened.len().unwrap(), 0);
    }

    #[test]
    fn legacy_store_without_meta_requires_migrate() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy");
        {
            let model = Model::open(&path).unwrap();
            model
                .add(Triple::new(
                    terms::named_node("https://example.com/s").unwrap(),
                    terms::named_node("https://example.com/p").unwrap(),
                    Literal::new_simple_literal("x"),
                ))
                .unwrap();
        }
        let disk = DiskStore::open(&path).unwrap();
        disk.quads.remove(META_KEY.as_bytes()).unwrap();
        disk.sync().unwrap();
        let err = Model::open(&path);
        assert!(matches!(err, Err(Error::Unsupported(_))));
        let migrated = Model::migrate_legacy_store(&path).unwrap();
        assert_eq!(migrated.len().unwrap(), 1);
        let disk = DiskStore::open(&path).unwrap();
        assert_eq!(disk.read_format_version().unwrap(), Some(FORMAT_VERSION));
    }
}
