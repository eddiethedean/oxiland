//! Storage backends, open options, and capability reporting.
//!
//! The backend identities, Cargo feature names, capability shape, and layout
//! reader policy in this module are the matrix frozen for Oxiland 1.0 by
//! ADR-024. The physical database directories are not interchangeable; use
//! standards RDF or [`crate::Model::copy_to`] to migrate between engines.

#[cfg_attr(
    not(any(
        feature = "storage-fjall",
        feature = "storage-redb",
        feature = "storage-rocksdb",
        feature = "storage-sqlite",
        feature = "storage-lmdb"
    )),
    allow(dead_code)
)]
mod backend_marker;
mod durable;
#[cfg(feature = "storage-fjall")]
mod fjall;
#[cfg_attr(
    not(any(
        feature = "storage-fjall",
        feature = "storage-redb",
        feature = "storage-rocksdb",
        feature = "storage-sqlite",
        feature = "storage-lmdb"
    )),
    allow(dead_code)
)]
mod format_v1;
#[cfg(feature = "storage-lmdb")]
mod lmdb;
#[cfg(feature = "storage-redb")]
mod redb;
#[cfg(feature = "storage-rocksdb")]
mod rocksdb;
#[cfg(feature = "storage-sqlite")]
mod sqlite;

pub(crate) use durable::{DurableStore, DurableStoreOps};
pub(crate) use format_v1::stored_matching_quad;

use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// Supported Oxiland storage backends.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum StorageBackend {
    /// In-memory Oxigraph store (`Model::new`).
    Memory,
    /// Durable Fjall keyspace plus Oxigraph working set (`Model::open`).
    Fjall,
    /// Durable redb table store (feature `storage-redb`).
    Redb,
    /// Durable RocksDB store (feature `storage-rocksdb`).
    RocksDb,
    /// Durable SQLite store (feature `storage-sqlite`).
    Sqlite,
    /// Durable LMDB store via heed (feature `storage-lmdb`).
    Lmdb,
}

/// Reader commitment for a backend's physical layout.
///
/// This is deliberately small and closed for the 1.0 storage contract. A
/// durable backend uses an engine-specific Oxiland format-v1 layout and keeps
/// a reader/export path available for every layout it has advertised. Memory
/// has no persistent layout to read.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LayoutReaderPolicy {
    /// The backend has no persistent files.
    None,
    /// The first-party adapter reads and exports its Oxiland format-v1 layout.
    FormatV1,
}

/// Frozen metadata for one supported storage backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBackendDescriptor {
    /// Typed backend identity.
    pub backend: StorageBackend,
    /// Canonical name accepted by [`StorageBackend::from_name`].
    pub name: &'static str,
    /// Cargo feature required for this backend, or `None` for memory.
    pub cargo_feature: Option<&'static str>,
    /// Whether the adapter is present in this build.
    pub compiled: bool,
    /// Whether data survives process restart.
    pub durable: bool,
    /// Physical-layout reader/export commitment.
    pub layout_reader: LayoutReaderPolicy,
}

const SUPPORTED_BACKENDS: [StorageBackend; 6] = [
    StorageBackend::Memory,
    StorageBackend::Fjall,
    StorageBackend::Redb,
    StorageBackend::RocksDb,
    StorageBackend::Sqlite,
    StorageBackend::Lmdb,
];

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
            Self::Redb => "redb",
            Self::RocksDb => "rocksdb",
            Self::Sqlite => "sqlite",
            Self::Lmdb => "lmdb",
        }
    }

    /// Whether this backend is compiled into the current build.
    #[must_use]
    pub const fn is_compiled(self) -> bool {
        match self {
            Self::Memory => true,
            Self::Fjall => cfg!(feature = "storage-fjall"),
            Self::Redb => cfg!(feature = "storage-redb"),
            Self::RocksDb => cfg!(feature = "storage-rocksdb"),
            Self::Sqlite => cfg!(feature = "storage-sqlite"),
            Self::Lmdb => cfg!(feature = "storage-lmdb"),
        }
    }

    /// Returns the frozen 1.0 descriptor for this backend.
    #[must_use]
    pub const fn descriptor(self) -> StorageBackendDescriptor {
        StorageBackendDescriptor {
            backend: self,
            name: self.name(),
            cargo_feature: match self {
                Self::Memory => None,
                Self::Fjall => Some("storage-fjall"),
                Self::Redb => Some("storage-redb"),
                Self::RocksDb => Some("storage-rocksdb"),
                Self::Sqlite => Some("storage-sqlite"),
                Self::Lmdb => Some("storage-lmdb"),
            },
            compiled: self.is_compiled(),
            durable: !matches!(self, Self::Memory),
            layout_reader: if matches!(self, Self::Memory) {
                LayoutReaderPolicy::None
            } else {
                LayoutReaderPolicy::FormatV1
            },
        }
    }

    /// Resolves a backend name.
    pub fn from_name(name: &str) -> Result<Self> {
        let normalized = name.trim().to_ascii_lowercase();
        let backend = match normalized.as_str() {
            "memory" | "mem" => Self::Memory,
            "fjall" => Self::Fjall,
            "redb" => Self::Redb,
            "rocksdb" => Self::RocksDb,
            "sqlite" => Self::Sqlite,
            "lmdb" => Self::Lmdb,
            other if KNOWN_OPTIONAL_BACKENDS.contains(&other) => {
                return Err(Error::Unsupported(format!(
                    "storage backend '{other}' is known but not compiled into this build"
                )));
            }
            other if LEGACY_REDLAND_BACKENDS.contains(&other) => {
                return Err(Error::Unsupported(format!(
                    "legacy Redland storage backend '{name}' is unsupported; export to N-Quads and use a supported Oxiland backend (see docs/design/0.4-legacy-storage.md)"
                )));
            }
            other => {
                return Err(Error::Unsupported(format!(
                    "storage backend '{other}' is not recognized"
                )));
            }
        };
        if backend != Self::Memory && !backend.is_compiled() {
            return Err(Error::Unsupported(format!(
                "storage backend '{}' is known but not compiled into this build",
                backend.name()
            )));
        }
        Ok(backend)
    }
}

/// All backend identities supported by the frozen 1.0 matrix.
///
/// Unlike [`compiled_backends`], this list is feature-independent. Use each
/// descriptor's [`StorageBackendDescriptor::compiled`] field for discovery
/// without losing the identity of an adapter disabled in the current build.
#[must_use]
pub fn supported_backends() -> impl ExactSizeIterator<Item = StorageBackendDescriptor> {
    SUPPORTED_BACKENDS
        .iter()
        .copied()
        .map(StorageBackend::descriptor)
}

/// Backends compiled into this build.
#[must_use]
pub fn compiled_backends() -> &'static [StorageBackend] {
    // Built once; order is stable for discovery APIs.
    static COMPILED: std::sync::LazyLock<Vec<StorageBackend>> = std::sync::LazyLock::new(|| {
        let mut backends = vec![StorageBackend::Memory];
        if cfg!(feature = "storage-fjall") {
            backends.push(StorageBackend::Fjall);
        }
        if cfg!(feature = "storage-redb") {
            backends.push(StorageBackend::Redb);
        }
        if cfg!(feature = "storage-rocksdb") {
            backends.push(StorageBackend::RocksDb);
        }
        if cfg!(feature = "storage-sqlite") {
            backends.push(StorageBackend::Sqlite);
        }
        if cfg!(feature = "storage-lmdb") {
            backends.push(StorageBackend::Lmdb);
        }
        backends
    });
    COMPILED.as_slice()
}

/// Returns whether `name` is a recognized backend identity (compiled or not).
#[must_use]
pub fn is_known_backend_name(name: &str) -> bool {
    let normalized = name.trim().to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "memory" | "mem" | "fjall" | "redb" | "rocksdb" | "sqlite" | "lmdb"
    ) || KNOWN_OPTIONAL_BACKENDS.contains(&normalized.as_str())
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

    /// Opens or creates a redb store at `path`.
    #[must_use]
    pub fn redb(path: impl AsRef<Path>) -> Self {
        Self::new(StorageBackend::Redb, path)
    }

    /// Opens or creates a RocksDB store at `path`.
    #[must_use]
    pub fn rocksdb(path: impl AsRef<Path>) -> Self {
        Self::new(StorageBackend::RocksDb, path)
    }

    /// Opens or creates a SQLite store at `path`.
    #[must_use]
    pub fn sqlite(path: impl AsRef<Path>) -> Self {
        Self::new(StorageBackend::Sqlite, path)
    }

    /// Opens or creates an LMDB store at `path`.
    #[must_use]
    pub fn lmdb(path: impl AsRef<Path>) -> Self {
        Self::new(StorageBackend::Lmdb, path)
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
    /// Capabilities for `backend` in the requested access mode.
    #[must_use]
    pub const fn for_backend(backend: StorageBackend, read_only: bool) -> Self {
        match backend {
            StorageBackend::Memory => Self::memory(),
            StorageBackend::Fjall => Self::fjall(read_only),
            StorageBackend::Redb => Self::redb(read_only),
            StorageBackend::RocksDb => Self::rocksdb(read_only),
            StorageBackend::Sqlite => Self::sqlite(read_only),
            StorageBackend::Lmdb => Self::lmdb(read_only),
        }
    }

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

    /// Capabilities for a redb-backed model.
    #[must_use]
    pub const fn redb(read_only: bool) -> Self {
        Self {
            backend: StorageBackend::Redb,
            durable: true,
            transactions: !read_only,
            sync: true,
            clear: !read_only,
            read_only,
            bulk_load: !read_only,
        }
    }

    /// Capabilities for a RocksDB-backed model.
    #[must_use]
    pub const fn rocksdb(read_only: bool) -> Self {
        Self {
            backend: StorageBackend::RocksDb,
            durable: true,
            transactions: !read_only,
            sync: true,
            clear: !read_only,
            read_only,
            bulk_load: !read_only,
        }
    }

    /// Capabilities for a SQLite-backed model.
    #[must_use]
    pub const fn sqlite(read_only: bool) -> Self {
        Self {
            backend: StorageBackend::Sqlite,
            durable: true,
            transactions: !read_only,
            sync: true,
            clear: !read_only,
            read_only,
            bulk_load: !read_only,
        }
    }

    /// Capabilities for an LMDB-backed model.
    #[must_use]
    pub const fn lmdb(read_only: bool) -> Self {
        Self {
            backend: StorageBackend::Lmdb,
            durable: true,
            transactions: !read_only,
            sync: true,
            clear: !read_only,
            read_only,
            bulk_load: !read_only,
        }
    }
}
