//! Sealed durable-store adapter (ADR-022). Not part of the public API.

use std::path::Path;
use std::sync::Arc;

use oxigraph::model::Quad;
use oxigraph::store::Store;

use crate::{Error, Result};

use super::{StorageBackend, StorageCapabilities};

/// Minimal interface required by the model's durable persistence layer.
///
/// Backend implementations depend on this sealed interface; model code depends
/// only on the interface and the type-erased handle below.
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

#[cfg(any(
    feature = "storage-fjall",
    feature = "storage-redb",
    feature = "storage-rocksdb",
    feature = "storage-sqlite",
    feature = "storage-lmdb"
))]
macro_rules! impl_durable_adapter {
    ($adapter:path) => {
        impl DurableStoreOps for $adapter {
            fn backend_id(&self) -> StorageBackend {
                <$adapter>::backend_id(self)
            }

            fn ensure_format_v1(&self, path: &Path, allow_init: bool) -> Result<()> {
                <$adapter>::ensure_format_v1(self, path, allow_init)
            }

            fn migrate_legacy_to_v1(&self) -> Result<()> {
                <$adapter>::migrate_legacy_to_v1(self)
            }

            fn load_into(&self, store: &Store) -> Result<()> {
                <$adapter>::load_into(self, store)
            }

            fn sync(&self) -> Result<()> {
                <$adapter>::sync(self)
            }

            fn insert_quad(&self, quad: &Quad) -> Result<()> {
                <$adapter>::insert(self, quad)
            }

            fn remove_rdf_equal(&self, quad: &Quad) -> Result<()> {
                <$adapter>::remove_rdf_equal(self, quad)
            }

            fn clear_quads(&self) -> Result<()> {
                <$adapter>::clear_quads(self)
            }

            fn replace_all_from_store(&self, store: &Store) -> Result<()> {
                <$adapter>::replace_all_from_store(self, store)
            }

            fn capabilities(&self, read_only: bool) -> StorageCapabilities {
                <$adapter>::capabilities(self, read_only)
            }
        }
    };
}

#[cfg(feature = "storage-fjall")]
impl_durable_adapter!(super::fjall::FjallStore);
#[cfg(feature = "storage-redb")]
impl_durable_adapter!(super::redb::RedbStore);
#[cfg(feature = "storage-rocksdb")]
impl_durable_adapter!(super::rocksdb::RocksDbStore);
#[cfg(feature = "storage-sqlite")]
impl_durable_adapter!(super::sqlite::SqliteStore);
#[cfg(feature = "storage-lmdb")]
impl_durable_adapter!(super::lmdb::LmdbStore);

/// Cloneable, type-erased durable backend handle.
///
/// Dynamic dispatch keeps backend-specific branching at construction time.
/// Operations remain closed over the narrow DurableStoreOps interface, so a
/// new adapter does not require another match arm in every model operation.
#[derive(Clone)]
pub(crate) struct DurableStore(Arc<dyn DurableStoreOps>);

impl DurableStore {
    #[cfg(any(
        feature = "storage-fjall",
        feature = "storage-redb",
        feature = "storage-rocksdb",
        feature = "storage-sqlite",
        feature = "storage-lmdb"
    ))]
    fn from_adapter(adapter: impl DurableStoreOps + 'static) -> Self {
        Self(Arc::new(adapter))
    }

    /// Opens a durable store for the backend at the given path.
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
            StorageBackend::Redb => Ok(Self::from_adapter(
                super::redb::RedbStore::open_with_create(path, create)?,
            )),
            #[cfg(not(feature = "storage-redb"))]
            StorageBackend::Redb => Err(uncompiled("redb")),
            #[cfg(feature = "storage-rocksdb")]
            StorageBackend::RocksDb => Ok(Self::from_adapter(
                super::rocksdb::RocksDbStore::open_with_create(path, create)?,
            )),
            #[cfg(not(feature = "storage-rocksdb"))]
            StorageBackend::RocksDb => Err(uncompiled("rocksdb")),
            #[cfg(feature = "storage-sqlite")]
            StorageBackend::Sqlite => Ok(Self::from_adapter(
                super::sqlite::SqliteStore::open_with_create(path, create)?,
            )),
            #[cfg(not(feature = "storage-sqlite"))]
            StorageBackend::Sqlite => Err(uncompiled("sqlite")),
            #[cfg(feature = "storage-lmdb")]
            StorageBackend::Lmdb => Ok(Self::from_adapter(
                super::lmdb::LmdbStore::open_with_create(path, create)?,
            )),
            #[cfg(not(feature = "storage-lmdb"))]
            StorageBackend::Lmdb => Err(uncompiled("lmdb")),
        }
    }

    #[cfg(feature = "storage-fjall")]
    fn open_fjall(path: &Path, create: bool) -> Result<Self> {
        use super::backend_marker::{reject_foreign_layout, write_backend_marker};
        reject_foreign_layout(path, StorageBackend::Fjall)?;
        let store = super::fjall::FjallStore::open_with_create(path, create)?;
        write_backend_marker(path, StorageBackend::Fjall)?;
        Ok(Self::from_adapter(store))
    }

    /// Returns whether path looks like an existing store for backend.
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
        self.0.backend_id()
    }

    fn ensure_format_v1(&self, path: &Path, allow_init: bool) -> Result<()> {
        self.0.ensure_format_v1(path, allow_init)
    }

    fn migrate_legacy_to_v1(&self) -> Result<()> {
        self.0.migrate_legacy_to_v1()
    }

    fn load_into(&self, store: &Store) -> Result<()> {
        self.0.load_into(store)
    }

    fn sync(&self) -> Result<()> {
        self.0.sync()
    }

    fn insert_quad(&self, quad: &Quad) -> Result<()> {
        self.0.insert_quad(quad)
    }

    fn remove_rdf_equal(&self, quad: &Quad) -> Result<()> {
        self.0.remove_rdf_equal(quad)
    }

    fn clear_quads(&self) -> Result<()> {
        self.0.clear_quads()
    }

    fn replace_all_from_store(&self, store: &Store) -> Result<()> {
        self.0.replace_all_from_store(store)
    }

    fn capabilities(&self, read_only: bool) -> StorageCapabilities {
        self.0.capabilities(read_only)
    }
}
