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
    pub(crate) fn replace_all_from_store(&self, store: &Store) -> Result<()> {
        use std::collections::HashSet;

        let mut keep = HashSet::new();
        for item in store.iter() {
            let quad = item.map_err(|error| Error::Storage(error.to_string()))?;
            let key = quad_key(&quad);
            self.quads
                .insert(key.as_bytes(), [])
                .map_err(|error| Error::Storage(error.to_string()))?;
            keep.insert(key);
        }

        let mut orphans = Vec::new();
        for entry in self.quads.iter() {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            let key = std::str::from_utf8(&key).map_err(|error| {
                Error::Storage(format!("persisted quad key was not UTF-8: {error}"))
            })?;
            if !keep.contains(key) {
                orphans.push(key.to_owned());
            }
        }
        for key in orphans {
            self.quads
                .remove(key.as_bytes())
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        self.keyspace
            .persist(PersistMode::SyncAll)
            .map_err(|error| Error::Storage(error.to_string()))
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
}
