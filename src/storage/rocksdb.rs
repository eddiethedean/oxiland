//! RocksDB durable adapter (SB-05 / 0.9).

use std::path::Path;
use std::sync::Arc;

use oxigraph::model::Quad;
use oxigraph::store::Store;
use rocksdb::{DB, Options, WriteBatch};

use crate::{Error, Result};

use super::backend_marker::{reject_foreign_layout, write_backend_marker};
use super::format_v1::{
    FORMAT_OXILAND, FORMAT_VERSION, META_KEY, parse_format_version, parse_quad, quad_key,
    quads_rdf_equal,
};
use super::{StorageBackend, StorageCapabilities};

/// Durable quad storage backed by RocksDB.
#[derive(Clone)]
pub(crate) struct RocksDbStore {
    db: Arc<DB>,
}

impl RocksDbStore {
    pub(crate) fn backend_id(&self) -> StorageBackend {
        StorageBackend::RocksDb
    }

    pub(crate) fn capabilities(&self, read_only: bool) -> StorageCapabilities {
        StorageCapabilities::rocksdb(read_only)
    }

    pub(crate) fn open_with_create(path: &Path, create: bool) -> Result<Self> {
        reject_foreign_layout(path, StorageBackend::RocksDb)?;
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
        } else if !looks_like_rocksdb_store(path) {
            return Err(Error::OpenStore {
                path: path.to_owned(),
                message:
                    "path is not an existing rocksdb/Oxiland store and OpenOptions::create(false)"
                        .into(),
            });
        }

        let mut opts = Options::default();
        opts.create_if_missing(create);
        let db = DB::open(&opts, path).map_err(|error| Error::OpenStore {
            path: path.to_owned(),
            message: error.to_string(),
        })?;
        write_backend_marker(path, StorageBackend::RocksDb)?;
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
            "{{\"format_version\":{FORMAT_VERSION},\"oxiland\":\"{FORMAT_OXILAND}\",\"backend\":\"rocksdb\"}}"
        );
        self.db
            .put(META_KEY.as_bytes(), meta.as_bytes())
            .map_err(|error| Error::Storage(error.to_string()))
    }

    fn read_format_version(&self) -> Result<Option<u32>> {
        match self
            .db
            .get(META_KEY.as_bytes())
            .map_err(|error| Error::Storage(error.to_string()))?
        {
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
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            let (key, _) = item.map_err(|error| Error::Storage(error.to_string()))?;
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
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            let (key, _) = item.map_err(|error| Error::Storage(error.to_string()))?;
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
        self.db
            .flush()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn insert(&self, quad: &Quad) -> Result<()> {
        let key = quad_key(quad);
        self.db
            .put(key.as_bytes(), [])
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn remove_rdf_equal(&self, quad: &Quad) -> Result<()> {
        let mut batch = WriteBatch::default();
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            let (key, _) = item.map_err(|error| Error::Storage(error.to_string()))?;
            let key = std::str::from_utf8(&key).map_err(|error| {
                Error::Storage(format!("persisted quad key was not UTF-8: {error}"))
            })?;
            if key == META_KEY {
                continue;
            }
            let stored = parse_quad(key)?;
            if quads_rdf_equal(&stored, quad)? {
                batch.delete(key.as_bytes());
            }
        }
        self.db
            .write(batch)
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn clear_quads(&self) -> Result<()> {
        let mut batch = WriteBatch::default();
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            let (key, _) = item.map_err(|error| Error::Storage(error.to_string()))?;
            let key = std::str::from_utf8(&key).map_err(|error| {
                Error::Storage(format!("persisted quad key was not UTF-8: {error}"))
            })?;
            if key != META_KEY {
                batch.delete(key.as_bytes());
            }
        }
        self.db
            .write(batch)
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn replace_all_from_store(&self, store: &Store) -> Result<()> {
        let mut replacement_keys = Vec::new();
        for quad in store.iter() {
            let quad = quad.map_err(|error| Error::Storage(error.to_string()))?;
            replacement_keys.push(quad_key(&quad));
        }
        let mut batch = WriteBatch::default();
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            let (key, _) = item.map_err(|error| Error::Storage(error.to_string()))?;
            if key.as_ref() != META_KEY.as_bytes() {
                batch.delete(key);
            }
        }
        for key in replacement_keys {
            batch.put(key.as_bytes(), []);
        }
        self.db
            .write(batch)
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.sync()
    }
}

pub(crate) fn looks_like_rocksdb_store(path: &Path) -> bool {
    path.is_dir() && path.join("CURRENT").exists()
}
