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
/// are appended after the last `/` in the base **path** (after `scheme://authority`),
/// not after a slash that belongs to `://`. Authority-only bases such as
/// `https://example.com` become `https://example.com/{relative}`.
///
/// Fragment and query components on the base are ignored when choosing the
/// directory prefix, so `#`-namespace bases are a poor fit—use
/// [`crate::utility::Namespace`] for those.
///
/// This is intentionally a Redland-shaped helper, not a full RFC 3986 resolver.
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
    let joined = format!("{}{relative}", directory_prefix(base));
    terms::named_node(joined)
}

/// Relativizes `iri` against `base` when `iri` is under `base`'s directory.
///
/// Returns `None` when relativization is not possible with the simple helper.
pub fn relativize_iri(base: &str, iri: &str) -> Result<Option<String>> {
    let base = terms::named_node(base)?.as_str().to_owned();
    let iri = terms::named_node(iri)?.as_str().to_owned();
    let prefix = directory_prefix(&base);
    if let Some(rest) = iri.strip_prefix(&prefix) {
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
///
/// Query and fragment components are stripped before decoding. Non-local hosts
/// other than `localhost` return [`Error::Unsupported`] on non-Windows targets;
/// on Windows they map to UNC paths (`\\host\share\…`).
pub fn file_uri_to_path(iri: impl AsRef<str>) -> Result<PathBuf> {
    let iri = iri.as_ref().trim();
    let rest = iri
        .strip_prefix("file:")
        .ok_or_else(|| Error::InvalidRdf(format!("expected file URI, got '{iri}'")))?;
    let (host, path_part) = if let Some(rest) = rest.strip_prefix("//") {
        if let Some(slash) = rest.find('/') {
            let host = &rest[..slash];
            (Some(host), &rest[slash..])
        } else {
            return Err(Error::InvalidRdf(format!(
                "file URI missing path component: '{iri}'"
            )));
        }
    } else if rest.starts_with('/') {
        (None, rest)
    } else {
        return Err(Error::InvalidRdf(format!("malformed file URI: '{iri}'")));
    };

    let path_part = strip_query_fragment(path_part);
    let decoded = percent_decode(path_part)?;

    if let Some(host) = host.filter(|h| !h.is_empty() && !h.eq_ignore_ascii_case("localhost")) {
        #[cfg(windows)]
        {
            // UNC: file://server/share/path → \\server\share\path
            let unc = format!("\\\\{host}{}", decoded.replace('/', "\\"));
            return Ok(PathBuf::from(unc));
        }
        #[cfg(not(windows))]
        {
            return Err(Error::Unsupported(format!(
                "file URI host '{host}' is unsupported; use localhost or empty host"
            )));
        }
    }

    #[cfg(windows)]
    {
        // file:///C:/Users/... → C:\Users\...
        if let Some(drive) = decoded.strip_prefix('/') {
            if drive.len() >= 2
                && drive.as_bytes()[0].is_ascii_alphabetic()
                && drive.as_bytes()[1] == b':'
            {
                return Ok(PathBuf::from(drive.replace('/', "\\")));
            }
        }
    }

    Ok(PathBuf::from(decoded))
}

/// Returns the directory prefix ending in `/` used for join/relativize.
fn directory_prefix(base: &str) -> String {
    let path_start = authority_end(base).unwrap_or(0);
    let path = &base[path_start..];
    let path = path.split(['?', '#']).next().unwrap_or(path);
    if let Some(rel) = path.rfind('/') {
        base[..path_start + rel + 1].to_owned()
    } else if path_start > 0 {
        // Authority-only (or path without '/'): append under the authority.
        format!("{}/", &base[..path_start])
    } else if let Some(idx) = base.rfind('/') {
        base[..=idx].to_owned()
    } else {
        format!("{base}/")
    }
}

/// Index just after `scheme://authority` (start of path), if present.
fn authority_end(iri: &str) -> Option<usize> {
    let rest = iri.find("://").map(|i| i + 3)?;
    match iri[rest..].find('/') {
        Some(rel) => Some(rest + rel),
        None => Some(iri.len()),
    }
}

fn looks_absolute(iri: &str) -> bool {
    iri.contains(':')
}

fn strip_query_fragment(path: &str) -> &str {
    path.split(['?', '#']).next().unwrap_or(path)
}

fn encode_file_path(path: &Path) -> Result<String> {
    let raw = path.to_str().ok_or_else(|| {
        Error::InvalidRdf(format!(
            "path '{}' is not valid UTF-8 (file IRIs require UTF-8)",
            path.display()
        ))
    })?;

    #[cfg(windows)]
    {
        let normalized = raw.replace('\\', "/");
        // UNC \\server\share\file
        if let Some(rest) = normalized.strip_prefix("//") {
            if let Some((server, share_path)) = rest.split_once('/') {
                if !server.is_empty() {
                    let mut out = format!("//{server}");
                    for segment in share_path.split('/') {
                        out.push('/');
                        encode_segment(segment, &mut out);
                    }
                    return Ok(out);
                }
            }
        }
        encode_local_path(&normalized)
    }
    #[cfg(not(windows))]
    {
        encode_local_path(raw)
    }
}

fn encode_local_path(raw: &str) -> Result<String> {
    let mut out = String::from("/");
    let trimmed = raw.trim_start_matches('/');
    for (i, segment) in trimmed.split('/').enumerate() {
        if i > 0 {
            out.push('/');
        }
        encode_segment(segment, &mut out);
    }
    Ok(out)
}

fn encode_segment(segment: &str, out: &mut String) {
    for byte in segment.bytes() {
        match byte {
            // Allow ':' so Windows drive letters stay `C:` (RFC 8089 style).
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b':' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_prefix_skips_scheme_slashes() {
        assert_eq!(
            directory_prefix("https://example.com/base/dir/"),
            "https://example.com/base/dir/"
        );
        assert_eq!(
            directory_prefix("https://example.com"),
            "https://example.com/"
        );
        assert_eq!(
            directory_prefix("https://example.com/"),
            "https://example.com/"
        );
    }
}
