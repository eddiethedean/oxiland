use std::path::Path;
use std::sync::Arc;

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::Quad;
use oxigraph::store::Store;
use redb::{Database, ReadableTable, TableDefinition};

use crate::{Error, Result};

const QUADS: TableDefinition<'_, &str, ()> = TableDefinition::new("oxiland_quads");

/// Durable quad storage backed by [redb](https://crates.io/crates/redb).
///
/// Oxigraph still provides the in-memory query engine; redb holds the durable
/// copy of every quad.
#[derive(Clone)]
pub(crate) struct DiskStore {
    db: Arc<Database>,
}

impl DiskStore {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|error| Error::OpenStore {
                    path: path.to_owned(),
                    message: error.to_string(),
                })?;
            }
        }

        let db = Database::create(path).map_err(|error| Error::OpenStore {
            path: path.to_owned(),
            message: error.to_string(),
        })?;

        Ok(Self { db: Arc::new(db) })
    }

    pub(crate) fn load_into(&self, store: &Store) -> Result<()> {
        let txn = self
            .db
            .begin_read()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let table = match txn.open_table(QUADS) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(()),
            Err(error) => return Err(Error::Storage(error.to_string())),
        };

        for entry in table
            .iter()
            .map_err(|error| Error::Storage(error.to_string()))?
        {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            let quad = parse_quad(key.value())?;
            store
                .insert(&quad)
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        Ok(())
    }

    pub(crate) fn insert(&self, quad: &Quad) -> Result<()> {
        let key = quad_key(quad);
        let txn = self
            .db
            .begin_write()
            .map_err(|error| Error::Storage(error.to_string()))?;
        {
            let mut table = txn
                .open_table(QUADS)
                .map_err(|error| Error::Storage(error.to_string()))?;
            table
                .insert(key.as_str(), ())
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        txn.commit()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn remove(&self, quad: &Quad) -> Result<()> {
        let key = quad_key(quad);
        let txn = self
            .db
            .begin_write()
            .map_err(|error| Error::Storage(error.to_string()))?;
        {
            let mut table = txn
                .open_table(QUADS)
                .map_err(|error| Error::Storage(error.to_string()))?;
            table
                .remove(key.as_str())
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        txn.commit()
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
