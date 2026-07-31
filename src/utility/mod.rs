//! URI, digest, Unicode, namespace, and vocabulary helpers (0.5).

mod digest;
mod namespace;
mod unicode;
mod uri;
pub mod vocab;

pub use digest::{DigestAlgorithm, digest_bytes, digest_hex, digest_path};
pub use namespace::Namespace;
pub use unicode::{normalize_nfc, normalize_nfkc};
pub use uri::{file_uri_to_path, join_iri, path_to_file_uri, relativize_iri, resolve_iri};
