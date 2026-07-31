//! redb durable adapter (SB-04 / 0.9).

use std::path::Path;
use std::sync::Arc;

use oxigraph::model::Quad;
use oxigraph::store::Store;
use redb::{Database, ReadableTable, TableDefinition};

use crate::{Error, Result};

use super::backend_marker::{reject_foreign_layout, write_backend_marker};
use super::format_v1::{
    FORMAT_OXILAND, FORMAT_VERSION, META_KEY, parse_format_version, parse_quad, quad_key,
    quads_rdf_equal,
};
use super::{StorageBackend, StorageCapabilities};

const QUADS: TableDefinition<'_, &str, &[u8]> = TableDefinition::new("oxiland_quads");
const DB_FILE: &str = "oxiland.redb";

/// Durable quad storage backed by redb.
#[derive(Clone)]
pub(crate) struct RedbStore {
    db: Arc<Database>,
}

impl RedbStore {
    pub(crate) fn backend_id(&self) -> StorageBackend {
        StorageBackend::Redb
    }

    pub(crate) fn capabilities(&self, read_only: bool) -> StorageCapabilities {
        StorageCapabilities::redb(read_only)
    }

    pub(crate) fn open_with_create(path: &Path, create: bool) -> Result<Self> {
        reject_foreign_layout(path, StorageBackend::Redb)?;
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
        } else if !looks_like_redb_store(path) {
            return Err(Error::OpenStore {
                path: path.to_owned(),
                message:
                    "path is not an existing redb/Oxiland store and OpenOptions::create(false)"
                        .into(),
            });
        }

        let db_path = path.join(DB_FILE);
        let db = if db_path.exists() {
            Database::open(&db_path)
        } else if create {
            Database::create(&db_path)
        } else {
            return Err(Error::OpenStore {
                path: path.to_owned(),
                message: "redb database file missing and create(false)".into(),
            });
        }
        .map_err(|error| Error::OpenStore {
            path: path.to_owned(),
            message: error.to_string(),
        })?;

        write_backend_marker(path, StorageBackend::Redb)?;
        Ok(Self { db: Arc::new(db) })
    }

    pub(crate) fn ensure_format_v1(&self, path: &Path, allow_init: bool) -> Result<()> {
        match self.read_format_version()? {
            Some(version) if version == FORMAT_VERSION => Ok(()),
            Some(version) => Err(Error::Unsupported(format!(
                "Oxiland on-disk format version {version} is not supported by this build (expected {FORMAT_VERSION})"
            ))),
            None => {
                if self.has_quad_keys()? {
                    Err(Error::Unsupported(format!(
                        "store at {} has quad keys without format metadata",
                        path.display()
                    )))
                } else if allow_init {
                    self.write_format_v1_meta()
                } else {
                    Err(Error::OpenStore {
                        path: path.to_owned(),
                        message: "store has no Oxiland format metadata and initialization is not allowed (read-only or create(false))"
                            .into(),
                    })
                }
            }
        }
    }

    pub(crate) fn migrate_legacy_to_v1(&self) -> Result<()> {
        if self.read_format_version()?.is_some() {
            return Ok(());
        }
        self.write_format_v1_meta()
    }

    fn write_format_v1_meta(&self) -> Result<()> {
        let meta = format!(
            "{{\"format_version\":{FORMAT_VERSION},\"oxiland\":\"{FORMAT_OXILAND}\",\"backend\":\"redb\"}}"
        );
        let write_txn = self
            .db
            .begin_write()
            .map_err(|error| Error::Storage(error.to_string()))?;
        {
            let mut table = write_txn
                .open_table(QUADS)
                .map_err(|error| Error::Storage(error.to_string()))?;
            table
                .insert(META_KEY, meta.as_bytes())
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    fn read_format_version(&self) -> Result<Option<u32>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let table = match read_txn.open_table(QUADS) {
            Ok(table) => table,
            Err(_) => return Ok(None),
        };
        match table
            .get(META_KEY)
            .map_err(|error| Error::Storage(error.to_string()))?
        {
            None => Ok(None),
            Some(value) => {
                let text = std::str::from_utf8(value.value()).map_err(|error| {
                    Error::Storage(format!("format metadata was not UTF-8: {error}"))
                })?;
                Ok(Some(parse_format_version(text)?))
            }
        }
    }

    fn has_quad_keys(&self) -> Result<bool> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let table = match read_txn.open_table(QUADS) {
            Ok(table) => table,
            Err(_) => return Ok(false),
        };
        for entry in table
            .iter()
            .map_err(|error| Error::Storage(error.to_string()))?
        {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            if key.value() != META_KEY {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(crate) fn load_into(&self, store: &Store) -> Result<()> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let table = match read_txn.open_table(QUADS) {
            Ok(table) => table,
            Err(_) => return Ok(()),
        };
        for entry in table
            .iter()
            .map_err(|error| Error::Storage(error.to_string()))?
        {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            let key = key.value();
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
        // redb commits are durable; explicit sync is a no-op success.
        Ok(())
    }

    pub(crate) fn insert(&self, quad: &Quad) -> Result<()> {
        let key = quad_key(quad);
        let write_txn = self
            .db
            .begin_write()
            .map_err(|error| Error::Storage(error.to_string()))?;
        {
            let mut table = write_txn
                .open_table(QUADS)
                .map_err(|error| Error::Storage(error.to_string()))?;
            table
                .insert(key.as_str(), [].as_slice())
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn remove_rdf_equal(&self, quad: &Quad) -> Result<()> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let table = read_txn
            .open_table(QUADS)
            .map_err(|error| Error::Storage(error.to_string()))?;
        let mut to_remove = Vec::new();
        for entry in table
            .iter()
            .map_err(|error| Error::Storage(error.to_string()))?
        {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            let key = key.value();
            if key == META_KEY {
                continue;
            }
            let stored = parse_quad(key)?;
            if quads_rdf_equal(&stored, quad)? {
                to_remove.push(key.to_owned());
            }
        }
        drop(table);
        drop(read_txn);
        if to_remove.is_empty() {
            return Ok(());
        }
        let write_txn = self
            .db
            .begin_write()
            .map_err(|error| Error::Storage(error.to_string()))?;
        {
            let mut table = write_txn
                .open_table(QUADS)
                .map_err(|error| Error::Storage(error.to_string()))?;
            for key in &to_remove {
                table
                    .remove(key.as_str())
                    .map_err(|error| Error::Storage(error.to_string()))?;
            }
        }
        write_txn
            .commit()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn clear_quads(&self) -> Result<()> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let table = read_txn
            .open_table(QUADS)
            .map_err(|error| Error::Storage(error.to_string()))?;
        let mut keys = Vec::new();
        for entry in table
            .iter()
            .map_err(|error| Error::Storage(error.to_string()))?
        {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            let key = key.value();
            if key != META_KEY {
                keys.push(key.to_owned());
            }
        }
        drop(table);
        drop(read_txn);
        let write_txn = self
            .db
            .begin_write()
            .map_err(|error| Error::Storage(error.to_string()))?;
        {
            let mut table = write_txn
                .open_table(QUADS)
                .map_err(|error| Error::Storage(error.to_string()))?;
            for key in &keys {
                table
                    .remove(key.as_str())
                    .map_err(|error| Error::Storage(error.to_string()))?;
            }
        }
        write_txn
            .commit()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn replace_all_from_store(&self, store: &Store) -> Result<()> {
        let mut keys = Vec::new();
        for quad in store.iter() {
            let quad = quad.map_err(|error| Error::Storage(error.to_string()))?;
            keys.push(quad_key(&quad));
        }
        let write_txn = self
            .db
            .begin_write()
            .map_err(|error| Error::Storage(error.to_string()))?;
        {
            let mut table = write_txn
                .open_table(QUADS)
                .map_err(|error| Error::Storage(error.to_string()))?;
            let mut existing = Vec::new();
            for entry in table
                .iter()
                .map_err(|error| Error::Storage(error.to_string()))?
            {
                let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
                if key.value() != META_KEY {
                    existing.push(key.value().to_owned());
                }
            }
            for key in &existing {
                table
                    .remove(key.as_str())
                    .map_err(|error| Error::Storage(error.to_string()))?;
            }
            for key in &keys {
                table
                    .insert(key.as_str(), [].as_slice())
                    .map_err(|error| Error::Storage(error.to_string()))?;
            }
        }
        write_txn
            .commit()
            .map_err(|error| Error::Storage(error.to_string()))
    }
}

pub(crate) fn looks_like_redb_store(path: &Path) -> bool {
    path.is_dir() && path.join(DB_FILE).exists()
}
