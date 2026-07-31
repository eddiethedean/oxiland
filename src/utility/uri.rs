//! Filename / IRI helpers.

use std::path::{Path, PathBuf};

use crate::terms::{self, NamedNode};
use crate::{Error, Result};

/// Creates a validated [`NamedNode`] from an IRI string.
pub fn resolve_iri(iri: impl AsRef<str>) -> Result<NamedNode> {
    terms::named_node(iri.as_ref())
}

/// Joins a base IRI and a relative reference using a simple path-append rule.
///
/// Absolute `relative` IRIs are returned as-is (validated). Relative references
/// are appended after the last `/` in `base` (or after the base string when no
/// slash is present). This is intentionally a Redland-shaped helper, not a full
/// RFC 3986 resolver.
pub fn join_iri(base: &str, relative: &str) -> Result<NamedNode> {
    let relative = relative.trim();
    if relative.is_empty() {
        return terms::named_node(base);
    }
    if looks_absolute(relative) {
        return terms::named_node(relative);
    }
    let base = base.trim();
    if base.is_empty() {
        return Err(Error::InvalidRdf(
            "cannot join relative IRI against an empty base".into(),
        ));
    }
    let joined = if let Some(idx) = base.rfind('/') {
        format!("{}{}", &base[..=idx], relative)
    } else {
        format!("{base}/{relative}")
    };
    terms::named_node(joined)
}

/// Relativizes `iri` against `base` when `iri` is under `base`'s directory.
///
/// Returns `None` when relativization is not possible with the simple helper.
pub fn relativize_iri(base: &str, iri: &str) -> Result<Option<String>> {
    let base = terms::named_node(base)?.as_str().to_owned();
    let iri = terms::named_node(iri)?.as_str().to_owned();
    let prefix = if let Some(idx) = base.rfind('/') {
        &base[..=idx]
    } else {
        return Ok(None);
    };
    if let Some(rest) = iri.strip_prefix(prefix) {
        if rest.is_empty() || rest.contains("://") {
            return Ok(None);
        }
        return Ok(Some(rest.to_owned()));
    }
    Ok(None)
}

/// Converts a filesystem path to a `file://` IRI.
pub fn path_to_file_uri(path: impl AsRef<Path>) -> Result<NamedNode> {
    let path = path.as_ref();
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(Error::Io)?.join(path)
    };
    let encoded = encode_file_path(&absolute)?;
    terms::named_node(format!("file://{encoded}"))
}

/// Converts a `file://` IRI to a filesystem path.
pub fn file_uri_to_path(iri: impl AsRef<str>) -> Result<PathBuf> {
    let iri = iri.as_ref().trim();
    let rest = iri
        .strip_prefix("file:")
        .ok_or_else(|| Error::InvalidRdf(format!("expected file URI, got '{iri}'")))?;
    let path_part = if let Some(rest) = rest.strip_prefix("//") {
        // file://host/path or file:///path
        if let Some(slash) = rest.find('/') {
            let host = &rest[..slash];
            if !host.is_empty() && !host.eq_ignore_ascii_case("localhost") {
                return Err(Error::Unsupported(format!(
                    "file URI host '{host}' is unsupported; use localhost or empty host"
                )));
            }
            &rest[slash..]
        } else {
            return Err(Error::InvalidRdf(format!(
                "file URI missing path component: '{iri}'"
            )));
        }
    } else if rest.starts_with('/') {
        rest
    } else {
        return Err(Error::InvalidRdf(format!("malformed file URI: '{iri}'")));
    };
    let decoded = percent_decode(path_part)?;
    Ok(PathBuf::from(decoded))
}

fn looks_absolute(iri: &str) -> bool {
    iri.contains(':')
}

fn encode_file_path(path: &Path) -> Result<String> {
    let raw = path.to_str().ok_or_else(|| {
        Error::InvalidRdf(format!("path '{}' is not valid UTF-8", path.display()))
    })?;
    let mut out = String::from("/");
    #[cfg(windows)]
    let raw = raw.replace('\\', "/");
    let trimmed = raw.trim_start_matches('/');
    for (i, segment) in trimmed.split('/').enumerate() {
        if i > 0 {
            out.push('/');
        }
        for byte in segment.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                    out.push(byte as char);
                }
                _ => out.push_str(&format!("%{byte:02X}")),
            }
        }
    }
    Ok(out)
}

fn percent_decode(input: &str) -> Result<String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hi = from_hex(bytes[i + 1])?;
                let lo = from_hex(bytes[i + 2])?;
                out.push((hi << 4) | lo);
                i += 3;
            }
            b'%' => {
                return Err(Error::InvalidRdf(
                    "truncated percent-escape in file URI".into(),
                ));
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).map_err(|error| {
        Error::InvalidRdf(format!("file URI path was not UTF-8 after decode: {error}"))
    })
}

fn from_hex(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(Error::InvalidRdf(
            "invalid percent-escape in file URI".into(),
        )),
    }
}
