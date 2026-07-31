# Utilities and logging

Oxiland provides Redland-shaped helpers under
[`oxiland::utility`](https://docs.rs/oxiland/latest/oxiland/utility/) plus
logging on [`World`](https://docs.rs/oxiland/latest/oxiland/struct.World.html)
(optional `tracing` feature).

## Digests

```rust
use oxiland::utility::{DigestAlgorithm, digest_hex};

assert_eq!(
    digest_hex(DigestAlgorithm::Sha256, b"abc"),
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
);
```

Supported names: `md5`, `sha1`, `sha256`. Others return `Error::Unsupported`.

## URI and file helpers

- `join_iri` / `relativize_iri` / `resolve_iri`
- `path_to_file_uri` / `file_uri_to_path`

`join_iri` is a path-append helper (not a full RFC 3986 resolver). Prefer
[`Namespace`](https://docs.rs/oxiland/latest/oxiland/utility/struct.Namespace.html)
for `#`-terminated vocabulary bases. Query and fragment on `file://` IRIs are
stripped by `file_uri_to_path`.

Malformed input returns `Error::InvalidRdf` or `Error::Unsupported`—utilities
do not panic.

## Unicode

`normalize_nfc` and `normalize_nfkc` wrap Unicode normalization forms.

## Namespaces and vocabulary

Namespace bases must end with `/`, `#`, or `:` (for example
`https://example.com/` or `http://example.org/ns#`).

```rust
use oxiland::utility::vocab::rdf;
use oxiland::utility::Namespace;

# fn main() -> oxiland::Result<()> {
let ex = Namespace::new("ex", "https://example.com/")?;
assert_eq!(ex.expand("alice")?.as_str(), "https://example.com/alice");
assert_eq!(
    rdf::type_().as_str(),
    "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
);
# Ok(())
# }
```

Curated vocab modules: `rdf`, `rdfs`, `xsd`, `owl`, and `dc` (Dublin Core
Terms `http://purl.org/dc/terms/`, not the older `elements/1.1/` namespace).

## Logging

```rust
use oxiland::{LogFacility, LogLevel, World};

let world = World::new();
world.set_log_level(LogLevel::Info);
world.set_log_handler(|record| eprintln!("{record}"));
world.log(LogLevel::Warn, LogFacility::Utility, "heads up");
```

Enable the Cargo feature `tracing` to also emit `tracing` events. Cloned
`World` values share the same handler and minimum log level.

## Hashes and lists

Redland hash/list types are replaced by `HashMap`, `Vec`, and Rust iterators.
See [migration from Redland](../evaluators/migration-from-redland.md) and
`cargo run --example std_replacements`.
