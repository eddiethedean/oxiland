//! Digest helpers (ADR-015).

use std::fs;
use std::path::Path;

use md5::{Digest, Md5};
use sha1::Sha1;
use sha2::Sha256;

use crate::{Error, Result};

/// Closed set of digest algorithms supported by Oxiland (ADR-015).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DigestAlgorithm {
    /// MD5 (legacy Redland digests; not for security).
    Md5,
    /// SHA-1 (legacy Redland digests; not for security).
    Sha1,
    /// SHA-256.
    Sha256,
}

impl DigestAlgorithm {
    /// Canonical short name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Md5 => "md5",
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
        }
    }

    /// Resolves a case-insensitive algorithm name.
    pub fn from_name(name: &str) -> Result<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "md5" => Ok(Self::Md5),
            "sha1" | "sha-1" => Ok(Self::Sha1),
            "sha256" | "sha-256" => Ok(Self::Sha256),
            other => Err(Error::Unsupported(format!(
                "digest algorithm '{other}' is unsupported; use md5, sha1, or sha256"
            ))),
        }
    }
}

/// Digests `data` and returns the raw digest bytes.
#[must_use]
pub fn digest_bytes(algorithm: DigestAlgorithm, data: &[u8]) -> Vec<u8> {
    match algorithm {
        DigestAlgorithm::Md5 => Md5::digest(data).to_vec(),
        DigestAlgorithm::Sha1 => Sha1::digest(data).to_vec(),
        DigestAlgorithm::Sha256 => Sha256::digest(data).to_vec(),
    }
}

/// Digests `data` and returns a lowercase hex string.
#[must_use]
pub fn digest_hex(algorithm: DigestAlgorithm, data: &[u8]) -> String {
    digest_bytes(algorithm, data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Digests the full contents of a filesystem path.
pub fn digest_path(algorithm: DigestAlgorithm, path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let path = path.as_ref();
    let data = fs::read(path).map_err(|error| {
        Error::Io(std::io::Error::new(
            error.kind(),
            format!("{}: {}", path.display(), error),
        ))
    })?;
    Ok(digest_bytes(algorithm, &data))
}
