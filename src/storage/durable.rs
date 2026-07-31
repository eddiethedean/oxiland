//! Sealed durable-store adapter (ADR-022). Not part of the public API.

use std::path::Path;

use oxigraph::model::Quad;
use oxigraph::store::Store;

use crate::{Error, Result};

use super::{StorageBackend, StorageCapabilities};

/// Sealed durable-store adapter (ADR-022). Not part of the public API.
pub(crate) trait DurableStoreOps: Send + Sync {
    fn backend_id(&self) -> StorageBackend;
    fn ensure_format_v1(&self, path: &Path, allow_init: bool) -> Result<()>;
    fn migrate_legacy_to_v1(&self) -> Result<()>;
    fn load_into(&self, store: &Store) -> Result<()>;
    fn sync(&self) -> Result<()>;
    fn insert_quad(&self, quad: &Quad) -> Result<()>;
    fn remove_rdf_equal(&self, quad: &Quad) -> Result<()>;
    fn clear_quads(&self) -> Result<()>;
    fn replace_all_from_store(&self, store: &Store) -> Result<()>;
    fn capabilities(&self, read_only: bool) -> StorageCapabilities;
}

/// Sealed durable backend handle.
#[derive(Clone)]
pub(crate) enum DurableStore {
    /// Uninhabited in practice; keeps no-default-feature builds well formed.
    #[cfg(not(any(
        feature = "storage-fjall",
        feature = "storage-redb",
        feature = "storage-rocksdb",
        feature = "storage-sqlite",
        feature = "storage-lmdb"
    )))]
    #[allow(dead_code)]
    Disabled,
    /// Fjall-backed format-v1 store.
    #[cfg(feature = "storage-fjall")]
    Fjall(super::fjall::FjallStore),
    /// redb-backed format-v1 store.
    #[cfg(feature = "storage-redb")]
    Redb(super::redb::RedbStore),
    /// RocksDB-backed format-v1 store.
    #[cfg(feature = "storage-rocksdb")]
    RocksDb(super::rocksdb::RocksDbStore),
    /// SQLite-backed format-v1 store.
    #[cfg(feature = "storage-sqlite")]
    Sqlite(super::sqlite::SqliteStore),
    /// LMDB-backed format-v1 store.
    #[cfg(feature = "storage-lmdb")]
    Lmdb(super::lmdb::LmdbStore),
}

impl DurableStore {
    /// Opens a durable store for `backend` at `path`.
    pub(crate) fn open(backend: StorageBackend, path: &Path, create: bool) -> Result<Self> {
        #[cfg(not(any(
            feature = "storage-fjall",
            feature = "storage-redb",
            feature = "storage-rocksdb",
            feature = "storage-sqlite",
            feature = "storage-lmdb"
        )))]
        let _ = (path, create);
        match backend {
            StorageBackend::Memory => Err(Error::Unsupported(
                "memory backend has no durable store handle".into(),
            )),
            #[cfg(feature = "storage-fjall")]
            StorageBackend::Fjall => Self::open_fjall(path, create),
            #[cfg(not(feature = "storage-fjall"))]
            StorageBackend::Fjall => Err(uncompiled("fjall")),
            #[cfg(feature = "storage-redb")]
            StorageBackend::Redb => Ok(Self::Redb(super::redb::RedbStore::open_with_create(
                path, create,
            )?)),
            #[cfg(not(feature = "storage-redb"))]
            StorageBackend::Redb => Err(uncompiled("redb")),
            #[cfg(feature = "storage-rocksdb")]
            StorageBackend::RocksDb => Ok(Self::RocksDb(
                super::rocksdb::RocksDbStore::open_with_create(path, create)?,
            )),
            #[cfg(not(feature = "storage-rocksdb"))]
            StorageBackend::RocksDb => Err(uncompiled("rocksdb")),
            #[cfg(feature = "storage-sqlite")]
            StorageBackend::Sqlite => Ok(Self::Sqlite(
                super::sqlite::SqliteStore::open_with_create(path, create)?,
            )),
            #[cfg(not(feature = "storage-sqlite"))]
            StorageBackend::Sqlite => Err(uncompiled("sqlite")),
            #[cfg(feature = "storage-lmdb")]
            StorageBackend::Lmdb => Ok(Self::Lmdb(super::lmdb::LmdbStore::open_with_create(
                path, create,
            )?)),
            #[cfg(not(feature = "storage-lmdb"))]
            StorageBackend::Lmdb => Err(uncompiled("lmdb")),
        }
    }

    /// Opens a Fjall durable store at `path`.
    #[cfg(feature = "storage-fjall")]
    pub(crate) fn open_fjall(path: &Path, create: bool) -> Result<Self> {
        use super::backend_marker::{reject_foreign_layout, write_backend_marker};
        reject_foreign_layout(path, StorageBackend::Fjall)?;
        let store = super::fjall::FjallStore::open_with_create(path, create)?;
        write_backend_marker(path, StorageBackend::Fjall)?;
        Ok(Self::Fjall(store))
    }

    /// Returns whether `path` looks like an existing store for `backend`.
    pub(crate) fn looks_like_store(backend: StorageBackend, path: &Path) -> bool {
        #[cfg(not(any(
            feature = "storage-fjall",
            feature = "storage-redb",
            feature = "storage-rocksdb",
            feature = "storage-sqlite",
            feature = "storage-lmdb"
        )))]
        let _ = path;
        match backend {
            StorageBackend::Memory => false,
            #[cfg(feature = "storage-fjall")]
            StorageBackend::Fjall => super::fjall::looks_like_fjall_store(path),
            #[cfg(not(feature = "storage-fjall"))]
            StorageBackend::Fjall => false,
            #[cfg(feature = "storage-redb")]
            StorageBackend::Redb => super::redb::looks_like_redb_store(path),
            #[cfg(not(feature = "storage-redb"))]
            StorageBackend::Redb => false,
            #[cfg(feature = "storage-rocksdb")]
            StorageBackend::RocksDb => super::rocksdb::looks_like_rocksdb_store(path),
            #[cfg(not(feature = "storage-rocksdb"))]
            StorageBackend::RocksDb => false,
            #[cfg(feature = "storage-sqlite")]
            StorageBackend::Sqlite => super::sqlite::looks_like_sqlite_store(path),
            #[cfg(not(feature = "storage-sqlite"))]
            StorageBackend::Sqlite => false,
            #[cfg(feature = "storage-lmdb")]
            StorageBackend::Lmdb => super::lmdb::looks_like_lmdb_store(path),
            #[cfg(not(feature = "storage-lmdb"))]
            StorageBackend::Lmdb => false,
        }
    }
}

#[cfg_attr(
    all(
        feature = "storage-fjall",
        feature = "storage-redb",
        feature = "storage-rocksdb",
        feature = "storage-sqlite",
        feature = "storage-lmdb"
    ),
    allow(dead_code)
)]
fn uncompiled(name: &str) -> Error {
    Error::Unsupported(format!(
        "storage backend '{name}' is known but not compiled into this build"
    ))
}

impl DurableStoreOps for DurableStore {
    fn backend_id(&self) -> StorageBackend {
        match self {
            #[cfg(not(any(
                feature = "storage-fjall",
                feature = "storage-redb",
                feature = "storage-rocksdb",
                feature = "storage-sqlite",
                feature = "storage-lmdb"
            )))]
            Self::Disabled => unreachable!("durable backends are disabled"),
            #[cfg(feature = "storage-fjall")]
            Self::Fjall(store) => store.backend_id(),
            #[cfg(feature = "storage-redb")]
            Self::Redb(store) => store.backend_id(),
            #[cfg(feature = "storage-rocksdb")]
            Self::RocksDb(store) => store.backend_id(),
            #[cfg(feature = "storage-sqlite")]
            Self::Sqlite(store) => store.backend_id(),
            #[cfg(feature = "storage-lmdb")]
            Self::Lmdb(store) => store.backend_id(),
        }
    }

    fn ensure_format_v1(&self, path: &Path, allow_init: bool) -> Result<()> {
        #[cfg(not(any(
            feature = "storage-fjall",
            feature = "storage-redb",
            feature = "storage-rocksdb",
            feature = "storage-sqlite",
            feature = "storage-lmdb"
        )))]
        let _ = (path, allow_init);
        match self {
            #[cfg(not(any(
                feature = "storage-fjall",
                feature = "storage-redb",
                feature = "storage-rocksdb",
                feature = "storage-sqlite",
                feature = "storage-lmdb"
            )))]
            Self::Disabled => unreachable!("durable backends are disabled"),
            #[cfg(feature = "storage-fjall")]
            Self::Fjall(store) => store.ensure_format_v1(path, allow_init),
            #[cfg(feature = "storage-redb")]
            Self::Redb(store) => store.ensure_format_v1(path, allow_init),
            #[cfg(feature = "storage-rocksdb")]
            Self::RocksDb(store) => store.ensure_format_v1(path, allow_init),
            #[cfg(feature = "storage-sqlite")]
            Self::Sqlite(store) => store.ensure_format_v1(path, allow_init),
            #[cfg(feature = "storage-lmdb")]
            Self::Lmdb(store) => store.ensure_format_v1(path, allow_init),
        }
    }

    fn migrate_legacy_to_v1(&self) -> Result<()> {
        match self {
            #[cfg(not(any(
                feature = "storage-fjall",
                feature = "storage-redb",
                feature = "storage-rocksdb",
                feature = "storage-sqlite",
                feature = "storage-lmdb"
            )))]
            Self::Disabled => unreachable!("durable backends are disabled"),
            #[cfg(feature = "storage-fjall")]
            Self::Fjall(store) => store.migrate_legacy_to_v1(),
            #[cfg(feature = "storage-redb")]
            Self::Redb(store) => store.migrate_legacy_to_v1(),
            #[cfg(feature = "storage-rocksdb")]
            Self::RocksDb(store) => store.migrate_legacy_to_v1(),
            #[cfg(feature = "storage-sqlite")]
            Self::Sqlite(store) => store.migrate_legacy_to_v1(),
            #[cfg(feature = "storage-lmdb")]
            Self::Lmdb(store) => store.migrate_legacy_to_v1(),
        }
    }

    fn load_into(&self, store: &Store) -> Result<()> {
        #[cfg(not(any(
            feature = "storage-fjall",
            feature = "storage-redb",
            feature = "storage-rocksdb",
            feature = "storage-sqlite",
            feature = "storage-lmdb"
        )))]
        let _ = store;
        match self {
            #[cfg(not(any(
                feature = "storage-fjall",
                feature = "storage-redb",
                feature = "storage-rocksdb",
                feature = "storage-sqlite",
                feature = "storage-lmdb"
            )))]
            Self::Disabled => unreachable!("durable backends are disabled"),
            #[cfg(feature = "storage-fjall")]
            Self::Fjall(disk) => disk.load_into(store),
            #[cfg(feature = "storage-redb")]
            Self::Redb(disk) => disk.load_into(store),
            #[cfg(feature = "storage-rocksdb")]
            Self::RocksDb(disk) => disk.load_into(store),
            #[cfg(feature = "storage-sqlite")]
            Self::Sqlite(disk) => disk.load_into(store),
            #[cfg(feature = "storage-lmdb")]
            Self::Lmdb(disk) => disk.load_into(store),
        }
    }

    fn sync(&self) -> Result<()> {
        match self {
            #[cfg(not(any(
                feature = "storage-fjall",
                feature = "storage-redb",
                feature = "storage-rocksdb",
                feature = "storage-sqlite",
                feature = "storage-lmdb"
            )))]
            Self::Disabled => unreachable!("durable backends are disabled"),
            #[cfg(feature = "storage-fjall")]
            Self::Fjall(store) => store.sync(),
            #[cfg(feature = "storage-redb")]
            Self::Redb(store) => store.sync(),
            #[cfg(feature = "storage-rocksdb")]
            Self::RocksDb(store) => store.sync(),
            #[cfg(feature = "storage-sqlite")]
            Self::Sqlite(store) => store.sync(),
            #[cfg(feature = "storage-lmdb")]
            Self::Lmdb(store) => store.sync(),
        }
    }

    fn insert_quad(&self, quad: &Quad) -> Result<()> {
        #[cfg(not(any(
            feature = "storage-fjall",
            feature = "storage-redb",
            feature = "storage-rocksdb",
            feature = "storage-sqlite",
            feature = "storage-lmdb"
        )))]
        let _ = quad;
        match self {
            #[cfg(not(any(
                feature = "storage-fjall",
                feature = "storage-redb",
                feature = "storage-rocksdb",
                feature = "storage-sqlite",
                feature = "storage-lmdb"
            )))]
            Self::Disabled => unreachable!("durable backends are disabled"),
            #[cfg(feature = "storage-fjall")]
            Self::Fjall(store) => store.insert(quad),
            #[cfg(feature = "storage-redb")]
            Self::Redb(store) => store.insert(quad),
            #[cfg(feature = "storage-rocksdb")]
            Self::RocksDb(store) => store.insert(quad),
            #[cfg(feature = "storage-sqlite")]
            Self::Sqlite(store) => store.insert(quad),
            #[cfg(feature = "storage-lmdb")]
            Self::Lmdb(store) => store.insert(quad),
        }
    }

    fn remove_rdf_equal(&self, quad: &Quad) -> Result<()> {
        #[cfg(not(any(
            feature = "storage-fjall",
            feature = "storage-redb",
            feature = "storage-rocksdb",
            feature = "storage-sqlite",
            feature = "storage-lmdb"
        )))]
        let _ = quad;
        match self {
            #[cfg(not(any(
                feature = "storage-fjall",
                feature = "storage-redb",
                feature = "storage-rocksdb",
                feature = "storage-sqlite",
                feature = "storage-lmdb"
            )))]
            Self::Disabled => unreachable!("durable backends are disabled"),
            #[cfg(feature = "storage-fjall")]
            Self::Fjall(store) => store.remove_rdf_equal(quad),
            #[cfg(feature = "storage-redb")]
            Self::Redb(store) => store.remove_rdf_equal(quad),
            #[cfg(feature = "storage-rocksdb")]
            Self::RocksDb(store) => store.remove_rdf_equal(quad),
            #[cfg(feature = "storage-sqlite")]
            Self::Sqlite(store) => store.remove_rdf_equal(quad),
            #[cfg(feature = "storage-lmdb")]
            Self::Lmdb(store) => store.remove_rdf_equal(quad),
        }
    }

    fn clear_quads(&self) -> Result<()> {
        match self {
            #[cfg(not(any(
                feature = "storage-fjall",
                feature = "storage-redb",
                feature = "storage-rocksdb",
                feature = "storage-sqlite",
                feature = "storage-lmdb"
            )))]
            Self::Disabled => unreachable!("durable backends are disabled"),
            #[cfg(feature = "storage-fjall")]
            Self::Fjall(store) => store.clear_quads(),
            #[cfg(feature = "storage-redb")]
            Self::Redb(store) => store.clear_quads(),
            #[cfg(feature = "storage-rocksdb")]
            Self::RocksDb(store) => store.clear_quads(),
            #[cfg(feature = "storage-sqlite")]
            Self::Sqlite(store) => store.clear_quads(),
            #[cfg(feature = "storage-lmdb")]
            Self::Lmdb(store) => store.clear_quads(),
        }
    }

    fn replace_all_from_store(&self, store: &Store) -> Result<()> {
        #[cfg(not(any(
            feature = "storage-fjall",
            feature = "storage-redb",
            feature = "storage-rocksdb",
            feature = "storage-sqlite",
            feature = "storage-lmdb"
        )))]
        let _ = store;
        match self {
            #[cfg(not(any(
                feature = "storage-fjall",
                feature = "storage-redb",
                feature = "storage-rocksdb",
                feature = "storage-sqlite",
                feature = "storage-lmdb"
            )))]
            Self::Disabled => unreachable!("durable backends are disabled"),
            #[cfg(feature = "storage-fjall")]
            Self::Fjall(disk) => disk.replace_all_from_store(store),
            #[cfg(feature = "storage-redb")]
            Self::Redb(disk) => disk.replace_all_from_store(store),
            #[cfg(feature = "storage-rocksdb")]
            Self::RocksDb(disk) => disk.replace_all_from_store(store),
            #[cfg(feature = "storage-sqlite")]
            Self::Sqlite(disk) => disk.replace_all_from_store(store),
            #[cfg(feature = "storage-lmdb")]
            Self::Lmdb(disk) => disk.replace_all_from_store(store),
        }
    }

    fn capabilities(&self, read_only: bool) -> StorageCapabilities {
        #[cfg(not(any(
            feature = "storage-fjall",
            feature = "storage-redb",
            feature = "storage-rocksdb",
            feature = "storage-sqlite",
            feature = "storage-lmdb"
        )))]
        let _ = read_only;
        match self {
            #[cfg(not(any(
                feature = "storage-fjall",
                feature = "storage-redb",
                feature = "storage-rocksdb",
                feature = "storage-sqlite",
                feature = "storage-lmdb"
            )))]
            Self::Disabled => unreachable!("durable backends are disabled"),
            #[cfg(feature = "storage-fjall")]
            Self::Fjall(store) => store.capabilities(read_only),
            #[cfg(feature = "storage-redb")]
            Self::Redb(store) => store.capabilities(read_only),
            #[cfg(feature = "storage-rocksdb")]
            Self::RocksDb(store) => store.capabilities(read_only),
            #[cfg(feature = "storage-sqlite")]
            Self::Sqlite(store) => store.capabilities(read_only),
            #[cfg(feature = "storage-lmdb")]
            Self::Lmdb(store) => store.capabilities(read_only),
        }
    }
}
