//! SQLite durable adapter (SB-05 / 0.9).

use std::path::Path;
use std::sync::{Arc, Mutex};

use oxigraph::model::Quad;
use oxigraph::store::Store;
use rusqlite::{Connection, OptionalExtension, params};

use crate::{Error, Result};

use super::backend_marker::{reject_foreign_layout, write_backend_marker};
use super::format_v1::{
    FORMAT_OXILAND, FORMAT_VERSION, META_KEY, parse_format_version, parse_quad, quad_key,
    quads_rdf_equal,
};
use super::{StorageBackend, StorageCapabilities};

const DB_FILE: &str = "oxiland.sqlite";

/// Durable quad storage backed by SQLite.
#[derive(Clone)]
pub(crate) struct SqliteStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStore {
    pub(crate) fn backend_id(&self) -> StorageBackend {
        StorageBackend::Sqlite
    }

    pub(crate) fn capabilities(&self, read_only: bool) -> StorageCapabilities {
        StorageCapabilities::sqlite(read_only)
    }

    pub(crate) fn open_with_create(path: &Path, create: bool) -> Result<Self> {
        reject_foreign_layout(path, StorageBackend::Sqlite)?;
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
        } else if !looks_like_sqlite_store(path) {
            return Err(Error::OpenStore {
                path: path.to_owned(),
                message:
                    "path is not an existing sqlite/Oxiland store and OpenOptions::create(false)"
                        .into(),
            });
        }

        let db_path = path.join(DB_FILE);
        if !db_path.exists() && !create {
            return Err(Error::OpenStore {
                path: path.to_owned(),
                message: "sqlite database file missing and create(false)".into(),
            });
        }
        let conn = Connection::open(&db_path).map_err(|error| Error::OpenStore {
            path: path.to_owned(),
            message: error.to_string(),
        })?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS oxiland_quads (
               key TEXT PRIMARY KEY NOT NULL,
               value BLOB NOT NULL
             );",
        )
        .map_err(|error| Error::OpenStore {
            path: path.to_owned(),
            message: error.to_string(),
        })?;
        write_backend_marker(path, StorageBackend::Sqlite)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
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
            "{{\"format_version\":{FORMAT_VERSION},\"oxiland\":\"{FORMAT_OXILAND}\",\"backend\":\"sqlite\"}}"
        );
        let conn = self
            .conn
            .lock()
            .map_err(|_| Error::Storage("sqlite lock poisoned".into()))?;
        conn.execute(
            "INSERT OR REPLACE INTO oxiland_quads(key, value) VALUES (?1, ?2)",
            params![META_KEY, meta.as_bytes()],
        )
        .map_err(|error| Error::Storage(error.to_string()))?;
        drop(conn);
        Ok(())
    }

    fn read_format_version(&self) -> Result<Option<u32>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| Error::Storage("sqlite lock poisoned".into()))?;
        let value: Option<Vec<u8>> = conn
            .query_row(
                "SELECT value FROM oxiland_quads WHERE key = ?1",
                params![META_KEY],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| Error::Storage(error.to_string()))?;
        drop(conn);
        match value {
            None => Ok(None),
            Some(bytes) => {
                let text = std::str::from_utf8(&bytes).map_err(|error| {
                    Error::Storage(format!("format metadata was not UTF-8: {error}"))
                })?;
                Ok(Some(parse_format_version(text)?))
            }
        }
    }

    fn has_quad_keys(&self) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| Error::Storage("sqlite lock poisoned".into()))?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM oxiland_quads WHERE key != ?1",
                params![META_KEY],
                |row| row.get(0),
            )
            .map_err(|error| Error::Storage(error.to_string()))?;
        drop(conn);
        Ok(count > 0)
    }

    pub(crate) fn load_into(&self, store: &Store) -> Result<()> {
        let keys = {
            let conn = self
                .conn
                .lock()
                .map_err(|_| Error::Storage("sqlite lock poisoned".into()))?;
            let mut stmt = conn
                .prepare("SELECT key FROM oxiland_quads WHERE key != ?1")
                .map_err(|error| Error::Storage(error.to_string()))?;
            let keys = stmt
                .query_map(params![META_KEY], |row| row.get::<_, String>(0))
                .map_err(|error| Error::Storage(error.to_string()))?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|error| Error::Storage(error.to_string()))?;
            drop(stmt);
            drop(conn);
            keys
        };
        for key in keys {
            let quad = parse_quad(&key)?;
            store
                .insert(&quad)
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        Ok(())
    }

    pub(crate) fn sync(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| Error::Storage("sqlite lock poisoned".into()))?;
        conn.execute_batch("PRAGMA wal_checkpoint(FULL);")
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn insert(&self, quad: &Quad) -> Result<()> {
        let key = quad_key(quad);
        let conn = self
            .conn
            .lock()
            .map_err(|_| Error::Storage("sqlite lock poisoned".into()))?;
        conn.execute(
            "INSERT OR REPLACE INTO oxiland_quads(key, value) VALUES (?1, X'')",
            params![key],
        )
        .map_err(|error| Error::Storage(error.to_string()))?;
        drop(conn);
        Ok(())
    }

    pub(crate) fn remove_rdf_equal(&self, quad: &Quad) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| Error::Storage("sqlite lock poisoned".into()))?;
        let mut stmt = conn
            .prepare("SELECT key FROM oxiland_quads WHERE key != ?1")
            .map_err(|error| Error::Storage(error.to_string()))?;
        let keys: Vec<String> = stmt
            .query_map(params![META_KEY], |row| row.get(0))
            .map_err(|error| Error::Storage(error.to_string()))?
            .collect::<std::result::Result<_, _>>()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let mut to_remove = Vec::new();
        for key in keys {
            let stored = parse_quad(&key)?;
            if quads_rdf_equal(&stored, quad)? {
                to_remove.push(key);
            }
        }
        drop(stmt);
        let tx = conn
            .unchecked_transaction()
            .map_err(|error| Error::Storage(error.to_string()))?;
        for key in to_remove {
            tx.execute("DELETE FROM oxiland_quads WHERE key = ?1", params![key])
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        tx.commit()
            .map_err(|error| Error::Storage(error.to_string()))?;
        drop(conn);
        Ok(())
    }

    pub(crate) fn clear_quads(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| Error::Storage("sqlite lock poisoned".into()))?;
        conn.execute(
            "DELETE FROM oxiland_quads WHERE key != ?1",
            params![META_KEY],
        )
        .map_err(|error| Error::Storage(error.to_string()))?;
        drop(conn);
        Ok(())
    }

    pub(crate) fn replace_all_from_store(&self, store: &Store) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| Error::Storage("sqlite lock poisoned".into()))?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|error| Error::Storage(error.to_string()))?;
        tx.execute(
            "DELETE FROM oxiland_quads WHERE key != ?1",
            params![META_KEY],
        )
        .map_err(|error| Error::Storage(error.to_string()))?;
        for quad in store.iter() {
            let quad = quad.map_err(|error| Error::Storage(error.to_string()))?;
            let key = quad_key(&quad);
            tx.execute(
                "INSERT INTO oxiland_quads(key, value) VALUES (?1, X'')",
                params![key],
            )
            .map_err(|error| Error::Storage(error.to_string()))?;
        }
        tx.commit()
            .map_err(|error| Error::Storage(error.to_string()))?;
        drop(conn);
        Ok(())
    }
}

pub(crate) fn looks_like_sqlite_store(path: &Path) -> bool {
    path.is_dir() && path.join(DB_FILE).exists()
}
