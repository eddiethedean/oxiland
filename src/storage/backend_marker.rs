//! Shared Oxiland backend identity marker (ADR-022 / 0.9).

use std::path::Path;

use crate::{Error, Result};

use super::StorageBackend;

/// Relative path of the backend identity marker inside a store directory.
pub(crate) const BACKEND_MARKER_FILE: &str = "oxiland.backend";

/// Writes `oxiland.backend` naming `backend` under `path`.
pub(crate) fn write_backend_marker(path: &Path, backend: StorageBackend) -> Result<()> {
    std::fs::create_dir_all(path).map_err(|error| Error::OpenStore {
        path: path.to_owned(),
        message: error.to_string(),
    })?;
    let marker = path.join(BACKEND_MARKER_FILE);
    std::fs::write(&marker, backend.name()).map_err(|error| Error::OpenStore {
        path: path.to_owned(),
        message: format!("failed to write backend marker: {error}"),
    })
}

/// Reads the backend named by `oxiland.backend`, if present.
pub(crate) fn read_backend_marker(path: &Path) -> Result<Option<String>> {
    let marker = path.join(BACKEND_MARKER_FILE);
    if !marker.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&marker).map_err(|error| Error::OpenStore {
        path: path.to_owned(),
        message: format!("failed to read backend marker: {error}"),
    })?;
    Ok(Some(text.trim().to_ascii_lowercase()))
}

/// Ensures `path` is either unmarked or already marked for `backend`.
///
/// Fails before mutating when another Oxiland backend owns the directory.
pub(crate) fn ensure_backend_marker(path: &Path, backend: StorageBackend) -> Result<()> {
    match read_backend_marker(path)? {
        None => Ok(()),
        Some(name) if name == backend.name() => Ok(()),
        Some(name) => Err(Error::OpenStore {
            path: path.to_owned(),
            message: format!(
                "store path is marked for backend '{name}', not '{}'; open with the matching backend or migrate via RDF export",
                backend.name()
            ),
        }),
    }
}

/// Rejects opening when another compiled backend's layout is visible.
pub(crate) fn reject_foreign_layout(path: &Path, backend: StorageBackend) -> Result<()> {
    ensure_backend_marker(path, backend)?;
    if let Some(other) = detect_layout_backend(path) {
        if other != backend {
            return Err(Error::OpenStore {
                path: path.to_owned(),
                message: format!(
                    "store path looks like a '{}' layout, not '{}'; wrong-backend open refused before mutation",
                    other.name(),
                    backend.name()
                ),
            });
        }
    }
    Ok(())
}

fn detect_layout_backend(path: &Path) -> Option<StorageBackend> {
    if !path.is_dir() {
        return None;
    }
    if path.join("oxiland.redb").exists() {
        return Some(StorageBackend::Redb);
    }
    if path.join("oxiland.sqlite").exists() {
        return Some(StorageBackend::Sqlite);
    }
    if path.join("CURRENT").exists() && path.join("IDENTITY").exists() {
        // RocksDB CURRENT + IDENTITY files.
        return Some(StorageBackend::RocksDb);
    }
    if path.join("data.mdb").exists() {
        return Some(StorageBackend::Lmdb);
    }
    const FJALL: &[&str] = &["keyspace", "partitions", "journals", "blobs", "version"];
    if FJALL.iter().any(|marker| path.join(marker).exists()) {
        return Some(StorageBackend::Fjall);
    }
    None
}
