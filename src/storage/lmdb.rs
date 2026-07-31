//! LMDB durable adapter via heed (SB-05 / 0.9).
//!
//! Heed requires `unsafe` when opening an environment because callers must
//! guarantee a single `Env` configuration per path. Oxiland opens one Env per
//! store directory and never shares the path with another Env in-process.

#![allow(unsafe_code)]

use std::fs::OpenOptions as FsOpenOptions;
use std::path::Path;
use std::sync::Arc;

use heed::types::{Bytes, Str};
use heed::{Database, Env, EnvOpenOptions};
use oxigraph::model::Quad;
use oxigraph::store::Store;

use crate::{Error, Result};

use super::backend_marker::{reject_foreign_layout, write_backend_marker};
use super::format_v1::{
    FORMAT_OXILAND, FORMAT_VERSION, META_KEY, parse_format_version, parse_quad, quad_key,
    quads_rdf_equal,
};
use super::{StorageBackend, StorageCapabilities};

const MAP_SIZE: usize = 1024 * 1024 * 256; // 256 MiB default map size

/// Durable quad storage backed by LMDB (heed).
#[derive(Clone)]
pub(crate) struct LmdbStore {
    env: Arc<Env>,
    db: Database<Str, Bytes>,
}

impl LmdbStore {
    pub(crate) fn backend_id(&self) -> StorageBackend {
        StorageBackend::Lmdb
    }

    pub(crate) fn capabilities(&self, read_only: bool) -> StorageCapabilities {
        StorageCapabilities::lmdb(read_only)
    }

    pub(crate) fn open_with_create(path: &Path, create: bool) -> Result<Self> {
        reject_foreign_layout(path, StorageBackend::Lmdb)?;
        if create {
            std::fs::create_dir_all(path).map_err(|error| Error::OpenStore {
                path: path.to_owned(),
                message: error.to_string(),
            })?;
            // Ensure the directory is writable for LMDB lock file creation.
            let _ = FsOpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path.join(".oxiland-lmdb-ready"))
                .map_err(|error| Error::OpenStore {
                    path: path.to_owned(),
                    message: error.to_string(),
                })?;
        } else if !path.exists() {
            return Err(Error::OpenStore {
                path: path.to_owned(),
                message: "path does not exist and OpenOptions::create(false)".into(),
            });
        } else if !looks_like_lmdb_store(path) {
            return Err(Error::OpenStore {
                path: path.to_owned(),
                message:
                    "path is not an existing lmdb/Oxiland store and OpenOptions::create(false)"
                        .into(),
            });
        }

        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(MAP_SIZE)
                .max_dbs(2)
                .open(path)
        }
        .map_err(|error| Error::OpenStore {
            path: path.to_owned(),
            message: error.to_string(),
        })?;

        let mut write_txn = env.write_txn().map_err(|error| Error::OpenStore {
            path: path.to_owned(),
            message: error.to_string(),
        })?;
        let db = env
            .create_database(&mut write_txn, Some("oxiland_quads"))
            .map_err(|error| Error::OpenStore {
                path: path.to_owned(),
                message: error.to_string(),
            })?;
        write_txn.commit().map_err(|error| Error::OpenStore {
            path: path.to_owned(),
            message: error.to_string(),
        })?;

        write_backend_marker(path, StorageBackend::Lmdb)?;
        Ok(Self {
            env: Arc::new(env),
            db,
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
            "{{\"format_version\":{FORMAT_VERSION},\"oxiland\":\"{FORMAT_OXILAND}\",\"backend\":\"lmdb\"}}"
        );
        let mut txn = self
            .env
            .write_txn()
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.db
            .put(&mut txn, META_KEY, meta.as_bytes())
            .map_err(|error| Error::Storage(error.to_string()))?;
        txn.commit()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    fn read_format_version(&self) -> Result<Option<u32>> {
        let txn = self
            .env
            .read_txn()
            .map_err(|error| Error::Storage(error.to_string()))?;
        match self
            .db
            .get(&txn, META_KEY)
            .map_err(|error| Error::Storage(error.to_string()))?
        {
            None => Ok(None),
            Some(bytes) => {
                let text = std::str::from_utf8(bytes).map_err(|error| {
                    Error::Storage(format!("format metadata was not UTF-8: {error}"))
                })?;
                Ok(Some(parse_format_version(text)?))
            }
        }
    }

    fn has_quad_keys(&self) -> Result<bool> {
        let txn = self
            .env
            .read_txn()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let iter = self
            .db
            .iter(&txn)
            .map_err(|error| Error::Storage(error.to_string()))?;
        for entry in iter {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            if key != META_KEY {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(crate) fn load_into(&self, store: &Store) -> Result<()> {
        let txn = self
            .env
            .read_txn()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let iter = self
            .db
            .iter(&txn)
            .map_err(|error| Error::Storage(error.to_string()))?;
        for entry in iter {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
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
        self.env
            .force_sync()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn insert(&self, quad: &Quad) -> Result<()> {
        let key = quad_key(quad);
        let mut txn = self
            .env
            .write_txn()
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.db
            .put(&mut txn, &key, &[])
            .map_err(|error| Error::Storage(error.to_string()))?;
        txn.commit()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn remove_rdf_equal(&self, quad: &Quad) -> Result<()> {
        let txn = self
            .env
            .read_txn()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let mut to_remove = Vec::new();
        let iter = self
            .db
            .iter(&txn)
            .map_err(|error| Error::Storage(error.to_string()))?;
        for entry in iter {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            if key == META_KEY {
                continue;
            }
            let stored = parse_quad(key)?;
            if quads_rdf_equal(&stored, quad)? {
                to_remove.push(key.to_owned());
            }
        }
        drop(txn);
        if to_remove.is_empty() {
            return Ok(());
        }
        let mut txn = self
            .env
            .write_txn()
            .map_err(|error| Error::Storage(error.to_string()))?;
        for key in &to_remove {
            self.db
                .delete(&mut txn, key)
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        txn.commit()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn clear_quads(&self) -> Result<()> {
        let txn = self
            .env
            .read_txn()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let mut keys = Vec::new();
        let iter = self
            .db
            .iter(&txn)
            .map_err(|error| Error::Storage(error.to_string()))?;
        for entry in iter {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            if key != META_KEY {
                keys.push(key.to_owned());
            }
        }
        drop(txn);
        let mut txn = self
            .env
            .write_txn()
            .map_err(|error| Error::Storage(error.to_string()))?;
        for key in &keys {
            self.db
                .delete(&mut txn, key)
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        txn.commit()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    pub(crate) fn replace_all_from_store(&self, store: &Store) -> Result<()> {
        let txn = self
            .env
            .read_txn()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let mut existing = Vec::new();
        let iter = self
            .db
            .iter(&txn)
            .map_err(|error| Error::Storage(error.to_string()))?;
        for entry in iter {
            let (key, _) = entry.map_err(|error| Error::Storage(error.to_string()))?;
            if key != META_KEY {
                existing.push(key.to_owned());
            }
        }
        drop(txn);

        let mut replacement_keys = Vec::new();
        for quad in store.iter() {
            let quad = quad.map_err(|error| Error::Storage(error.to_string()))?;
            replacement_keys.push(quad_key(&quad));
        }

        let mut txn = self
            .env
            .write_txn()
            .map_err(|error| Error::Storage(error.to_string()))?;
        for key in &existing {
            self.db
                .delete(&mut txn, key)
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        for key in &replacement_keys {
            self.db
                .put(&mut txn, key, &[])
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        txn.commit()
            .map_err(|error| Error::Storage(error.to_string()))
    }
}

pub(crate) fn looks_like_lmdb_store(path: &Path) -> bool {
    path.is_dir() && path.join("data.mdb").exists()
}
