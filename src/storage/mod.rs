//! Storage backends, open options, and capability reporting (0.4 / ADR-022).

mod durable;
mod fjall;
mod format_v1;

pub(crate) use durable::{DurableStore, DurableStoreOps};
pub(crate) use format_v1::stored_matching_quad;

use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// Supported Oxiland storage backends.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StorageBackend {
    /// In-memory Oxigraph store (`Model::new`).
    Memory,
    /// Durable Fjall keyspace plus Oxigraph working set (`Model::open`).
    Fjall,
}

const KNOWN_OPTIONAL_BACKENDS: &[&str] = &[
    "redb",
    "rocksdb",
    "sqlite",
    "lmdb",
    "sled",
    "leveldb",
    "mdbx",
    "surrealkv",
];

const LEGACY_REDLAND_BACKENDS: &[&str] = &[
    "hashes",
    "file",
    "mysql",
    "postgresql",
    "postgres",
    "tstore",
    "uri",
    "virtuoso",
];

impl StorageBackend {
    /// Canonical short name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Fjall => "fjall",
        }
    }

    /// Resolves a backend name.
    pub fn from_name(name: &str) -> Result<Self> {
        let normalized = name.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "memory" | "mem" => Ok(Self::Memory),
            "fjall" => Ok(Self::Fjall),
            other if KNOWN_OPTIONAL_BACKENDS.contains(&other) => Err(Error::Unsupported(format!(
                "storage backend '{other}' is known but not compiled into this build"
            ))),
            other if LEGACY_REDLAND_BACKENDS.contains(&other) => Err(Error::Unsupported(format!(
                "legacy Redland storage backend '{name}' is unsupported; export to N-Quads and use memory or fjall (see docs/design/0.4-legacy-storage.md)"
            ))),
            other => Err(Error::Unsupported(format!(
                "storage backend '{other}' is not recognized"
            ))),
        }
    }
}

/// Backends compiled into this build.
#[must_use]
pub fn compiled_backends() -> &'static [StorageBackend] {
    &[StorageBackend::Memory, StorageBackend::Fjall]
}

/// Returns whether `name` is a recognized backend identity (compiled or not).
#[must_use]
pub fn is_known_backend_name(name: &str) -> bool {
    let normalized = name.trim().to_ascii_lowercase();
    matches!(normalized.as_str(), "memory" | "mem" | "fjall")
        || KNOWN_OPTIONAL_BACKENDS.contains(&normalized.as_str())
        || LEGACY_REDLAND_BACKENDS.contains(&normalized.as_str())
}

/// Options for opening a model with a selected storage backend (ADR-006 / ADR-022).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenOptions {
    backend: StorageBackend,
    path: PathBuf,
    read_only: bool,
    create: bool,
}

impl OpenOptions {
    /// Opens a store for `backend` at `path` (read-write, create allowed).
    #[must_use]
    pub fn new(backend: StorageBackend, path: impl AsRef<Path>) -> Self {
        Self {
            backend,
            path: path.as_ref().to_owned(),
            read_only: false,
            create: true,
        }
    }

    /// Opens or creates a Fjall store at `path` (read-write, create allowed).
    #[must_use]
    pub fn fjall(path: impl AsRef<Path>) -> Self {
        Self::new(StorageBackend::Fjall, path)
    }

    /// Returns the configured backend.
    #[must_use]
    pub fn backend(&self) -> StorageBackend {
        self.backend
    }

    /// Returns the configured path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// When true, mutating APIs return [`Error::Unsupported`].
    #[must_use]
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// When false, opening a missing path fails instead of creating a store.
    #[must_use]
    pub fn create(mut self, create: bool) -> Self {
        self.create = create;
        self
    }

    /// Whether the store should reject writes.
    #[must_use]
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Whether missing paths may be created.
    #[must_use]
    pub fn can_create(&self) -> bool {
        self.create
    }
}

/// Capability bits for a model or backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageCapabilities {
    /// Backend kind.
    pub backend: StorageBackend,
    /// Quads survive process restart.
    pub durable: bool,
    /// [`crate::Model::transaction`] is available.
    pub transactions: bool,
    /// [`crate::Model::sync`] is meaningful (no-op success on memory).
    pub sync: bool,
    /// [`crate::Model::clear`] / `clear_graph` are available.
    pub clear: bool,
    /// Store was opened read-only.
    pub read_only: bool,
    /// Bulk / transactional load helpers are available.
    pub bulk_load: bool,
}

impl StorageCapabilities {
    /// Capabilities for an in-memory model.
    #[must_use]
    pub const fn memory() -> Self {
        Self {
            backend: StorageBackend::Memory,
            durable: false,
            transactions: true,
            sync: true,
            clear: true,
            read_only: false,
            bulk_load: true,
        }
    }

    /// Capabilities for a Fjall-backed model.
    #[must_use]
    pub const fn fjall(read_only: bool) -> Self {
        Self {
            backend: StorageBackend::Fjall,
            durable: true,
            transactions: !read_only,
            sync: true,
            clear: !read_only,
            read_only,
            bulk_load: !read_only,
        }
    }
}
