//! Storage backends, open options, and capability reporting (0.4).

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
        match name.trim().to_ascii_lowercase().as_str() {
            "memory" | "mem" => Ok(Self::Memory),
            "fjall" => Ok(Self::Fjall),
            "rocksdb" | "redb" => Err(Error::Unsupported(
                "storage backend was replaced by fjall; use Model::open / OpenOptions::fjall"
                    .into(),
            )),
            "hashes" | "file" | "mysql" | "postgresql" | "postgres" | "sqlite" | "tstore"
            | "uri" | "virtuoso" => Err(Error::Unsupported(format!(
                "legacy Redland storage backend '{name}' is unsupported; export to N-Quads and use memory or fjall (see docs/design/0.4-legacy-storage.md)"
            ))),
            other => Err(Error::Unsupported(format!(
                "storage backend '{other}' is not recognized"
            ))),
        }
    }
}

/// Options for opening a durable Fjall-backed model (ADR-006).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenOptions {
    path: PathBuf,
    read_only: bool,
    create: bool,
}

impl OpenOptions {
    /// Opens or creates a Fjall store at `path` (read-write, create allowed).
    #[must_use]
    pub fn fjall(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_owned(),
            read_only: false,
            create: true,
        }
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
