//! Sealed durable-store adapter (ADR-022). Not part of the public API.

use std::path::Path;

use oxigraph::model::Quad;
use oxigraph::store::Store;

use crate::Result;

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
    /// Fjall-backed format-v1 store.
    Fjall(super::fjall::FjallStore),
}

impl DurableStore {
    /// Opens a Fjall durable store at `path`.
    pub(crate) fn open_fjall(path: &Path, create: bool) -> Result<Self> {
        Ok(Self::Fjall(super::fjall::FjallStore::open_with_create(
            path, create,
        )?))
    }

    /// Returns whether `path` looks like an existing store for `backend`.
    pub(crate) fn looks_like_store(backend: StorageBackend, path: &Path) -> bool {
        match backend {
            StorageBackend::Fjall => super::fjall::looks_like_fjall_store(path),
            StorageBackend::Memory => false,
        }
    }
}

impl DurableStoreOps for DurableStore {
    fn backend_id(&self) -> StorageBackend {
        match self {
            Self::Fjall(store) => store.backend_id(),
        }
    }

    fn ensure_format_v1(&self, path: &Path, allow_init: bool) -> Result<()> {
        match self {
            Self::Fjall(store) => store.ensure_format_v1(path, allow_init),
        }
    }

    fn migrate_legacy_to_v1(&self) -> Result<()> {
        match self {
            Self::Fjall(store) => store.migrate_legacy_to_v1(),
        }
    }

    fn load_into(&self, store: &Store) -> Result<()> {
        match self {
            Self::Fjall(disk) => disk.load_into(store),
        }
    }

    fn sync(&self) -> Result<()> {
        match self {
            Self::Fjall(store) => store.sync(),
        }
    }

    fn insert_quad(&self, quad: &Quad) -> Result<()> {
        match self {
            Self::Fjall(store) => store.insert(quad),
        }
    }

    fn remove_rdf_equal(&self, quad: &Quad) -> Result<()> {
        match self {
            Self::Fjall(store) => store.remove_rdf_equal(quad),
        }
    }

    fn clear_quads(&self) -> Result<()> {
        match self {
            Self::Fjall(store) => store.clear_quads(),
        }
    }

    fn replace_all_from_store(&self, store: &Store) -> Result<()> {
        match self {
            Self::Fjall(disk) => disk.replace_all_from_store(store),
        }
    }

    fn capabilities(&self, read_only: bool) -> StorageCapabilities {
        match self {
            Self::Fjall(store) => store.capabilities(read_only),
        }
    }
}
