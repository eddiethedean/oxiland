#[cfg(test)]
use std::cell::Cell;
use std::path::Path;

use fjall::{Config, Keyspace, Partition, PartitionCreateOptions, PersistMode};
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::Quad;
use oxigraph::store::Store;

use crate::{Error, Result};

const QUADS_PARTITION: &str = "oxiland_quads";

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
/// copy of every quad.
#[derive(Clone)]
pub(crate) struct DiskStore {
    keyspace: Keyspace,
    quads: Partition,
}

impl DiskStore {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        std::fs::create_dir_all(path).map_err(|error| Error::OpenStore {
            path: path.to_owned(),
            message: error.to_string(),
        })?;

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

    pub(crate) fn load_into(&self, store: &Store) -> Result<()> {
        for entry in self.quads.iter() {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            let key = std::str::from_utf8(&key).map_err(|error| {
                Error::Storage(format!("persisted quad key was not UTF-8: {error}"))
            })?;
            let quad = parse_quad(key)?;
            store
                .insert(&quad)
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        Ok(())
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
        self.keyspace
            .persist(PersistMode::SyncAll)
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn remove(&self, quad: &Quad) -> Result<()> {
        let key = quad_key(quad);
        self.quads
            .remove(key.as_bytes())
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.keyspace
            .persist(PersistMode::SyncAll)
            .map_err(|error| Error::Storage(error.to_string()))
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
            current.insert(key.to_owned());
        }

        let to_insert: Vec<_> = desired.difference(&current).cloned().collect();
        let to_delete: Vec<_> = current.difference(&desired).cloned().collect();

        let mut inserted = Vec::new();
        for key in &to_insert {
            if let Err(error) = self.quads.insert(key.as_bytes(), []) {
                self.compensate_replace(&inserted, &[]);
                return Err(Error::Storage(error.to_string()));
            }
            inserted.push(key.clone());
        }

        #[cfg(test)]
        if DISK_REPLACE_FAULT.with(Cell::get) {
            self.compensate_replace(&inserted, &[]);
            return Err(Error::Storage(
                "injected disk replace fault after inserts".into(),
            ));
        }

        let mut deleted = Vec::new();
        for key in &to_delete {
            if let Err(error) = self.quads.remove(key.as_bytes()) {
                self.compensate_replace(&inserted, &deleted);
                return Err(Error::Storage(error.to_string()));
            }
            deleted.push(key.clone());
        }

        #[cfg(test)]
        if DISK_REPLACE_PERSIST_FAULT.with(Cell::get) {
            self.compensate_replace(&inserted, &deleted);
            return Err(Error::Storage(
                "injected disk replace fault before persist".into(),
            ));
        }

        if let Err(error) = self.keyspace.persist(PersistMode::SyncAll) {
            self.compensate_replace(&inserted, &deleted);
            let _ = self.keyspace.persist(PersistMode::SyncAll);
            return Err(Error::Storage(error.to_string()));
        }
        Ok(())
    }

    fn compensate_replace(&self, inserted: &[String], deleted: &[String]) {
        for key in deleted {
            let _ = self.quads.insert(key.as_bytes(), []);
        }
        for key in inserted {
            let _ = self.quads.remove(key.as_bytes());
        }
    }
}

fn quad_key(quad: &Quad) -> String {
    format!("{quad} .")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Model;
    use crate::terms::{self, Literal, Triple};
    use oxigraph::model::Quad;

    #[test]
    fn duplicate_insert_disk_fault_preserves_existing_quad() {
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

        DISK_INSERT_FAULT.with(|flag| flag.set(true));
        let err = model.insert_quad(quad).unwrap_err();
        DISK_INSERT_FAULT.with(|flag| flag.set(false));
        assert!(matches!(err, Error::Storage(_)));
        assert_eq!(model.len().unwrap(), 1);
        assert!(model.contains(statement.as_ref()).unwrap());
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
}
