# Oxiland

[![CI](https://github.com/eddiethedean/oxiland/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/oxiland/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/oxiland.svg)](https://crates.io/crates/oxiland)
[![API docs (docs.rs)](https://img.shields.io/docsrs/oxiland?label=API%20docs)](https://docs.rs/oxiland)
[![Guides (Read the Docs)](https://img.shields.io/readthedocs/oxiland?label=Guides)](https://oxiland.readthedocs.io/en/latest/)
[![MSRV](https://img.shields.io/crates/msrv/oxiland)](https://crates.io/crates/oxiland)
[![License](https://img.shields.io/crates/l/oxiland)](LICENSE-APACHE)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/eddiethedean/oxiland)

**Oxiland** is a safe Rust facade for Redland-shaped RDF workflows—models,
contexts, statement matching, SPARQL query/update/results, and stream-oriented
Turtle / N-Triples / N-Quads / TriG / RDF/XML I/O—backed by pinned
[Oxigraph](https://oxigraph.org/) 0.5.9 with `#![forbid(unsafe_code)]`.

Use it when you want Redland concepts and explicit unsupported/error behavior
without C ownership; use Oxigraph directly when you only need the engine API.
Version **0.5.0** covers the trusted core model, Redland-shaped RDF I/O,
SPARQL query/update/results, durable Fjall storage (format v1), utilities
(digests, URI/file/Unicode helpers, vocabulary), and World logging—with scoped
evidence in the [parity ledger](PARITY.md)—not C ABI/source compatibility or
full `librdf` accounting.

> [!IMPORTANT]
> Compatibility claims are evidence-scoped. See the
> [parity ledger](PARITY.md) and [0.5 report](docs/reports/0.5.md). Do not read
> “Redland-shaped” as drop-in C or ABI compatibility.

## When to use Oxiland

| Choose Oxiland when… | Prefer Oxigraph directly when… |
|---|---|
| You want Redland-like models, contexts, and I/O facades | You only need Oxigraph’s store/SPARQL/I/O API |
| You need explicit `Unsupported` / typed errors for missing features | You want every Oxigraph capability without a facade |
| You are migrating Redland *concepts* into safe Rust | You are starting a greenfield Oxigraph app |
| You care about inventory-backed compatibility evidence | You do not need Redland workflow mapping |

A longer comparison (including Sophia and native Redland) is in
[docs/evaluators/positioning.md](docs/evaluators/positioning.md).

## Current capabilities

| Capability | Status |
|---|---|
| RDF named nodes, blank nodes, literals, triples, and quads | Available through Oxigraph types |
| In-memory model | Available |
| Default-graph CRUD | Available |
| Named-graph/context CRUD and matching | Available |
| Partial statement matching | Available; streaming `StatementMatches` |
| SPARQL ASK / SELECT / CONSTRUCT / DESCRIBE | Available; streaming `QueryResults` |
| RDF parser and serializer facades | Available; Turtle, N-Triples, N-Quads, TriG, RDF/XML |
| Syntax discovery by name, MIME type, and extension | Available via `Syntax` |
| Persistent Fjall model | Available; format v1 via `Model::open` / `open_with` (ADR-006) |
| Transactions / sync / clear | Available; `Model::transaction`, `sync`, `clear` |
| SPARQL Update and results serialization | Available; XML/JSON/CSV/TSV + graph serialize helper |
| Digests / URI / Unicode / vocab helpers | Available; `oxiland::utility` |
| World logging | Available; handlers + optional `tracing` feature (ADR-014) |
| Python package (Pythonic PyPI API) | Planned for 0.7 |
| Full safe Rust Redland accounting | Planned for 0.6 |
| C source and ABI compatibility | Planned for 0.8–0.9 |

“Available” means the current public workflow is implemented and tested. It
does not imply full subsystem parity with Redland.

### Advertised RDF syntaxes

| Syntax | Name | Media type | Extension | Datasets |
|---|---|---|---|---|
| Turtle | `turtle` | `text/turtle` | `.ttl` | no |
| N-Triples | `ntriples` | `application/n-triples` | `.nt` | no |
| N-Quads | `nquads` | `application/n-quads` | `.nq` | yes |
| TriG | `trig` | `application/trig` | `.trig` | yes |
| RDF/XML | `rdfxml` | `application/rdf+xml` | `.rdf` | no |

Ambiguous aliases (`text/plain`, `application/xml`, `.txt`, `.xml`) and N3 /
JSON-LD return [`Error::Unsupported`](https://docs.rs/oxiland). Full
dispositions: [`compatibility/baseline/format-matrix.json`](compatibility/baseline/format-matrix.json).

## Requirements

- Rust **1.87** or newer (edition 2024; matches the Oxigraph 0.5.9 pin used for
  compatibility testing)

Install or update with [rustup](https://rustup.rs/):

```console
rustup update stable
rustc --version   # >= 1.87
```

## Installation

```toml
[dependencies]
oxiland = "0.5.0"
```

## Quick start

```rust
use oxiland::terms::{Literal, NamedNode, Triple};
use oxiland::{Model, Query, QueryResults};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::new()?;

    model.add(Triple::new(
        NamedNode::new("https://example.com/alice")?,
        NamedNode::new("https://example.com/name")?,
        Literal::new_simple_literal("Alice"),
    ))?;

    let result = Query::new("ASK { ?s ?p ?o }").execute(&model)?;
    assert!(matches!(result, QueryResults::Boolean(true)));

    Ok(())
}
```

`Model::add` returns `true` when it inserts a new statement and `false` when the
same statement already exists.

```console
cargo run --example quick_start
```

More workflows: [Getting started](docs/users/getting-started.md).

## Contexts and pattern matching

Redland contexts map to Oxigraph graph names:

```rust
use oxiland::terms::{GraphName, Literal, NamedNode, Triple};
use oxiland::{Model, StatementPattern};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::new()?;
    let subject = NamedNode::new("https://example.com/alice")?;
    let graph = NamedNode::new("https://example.com/people")?;

    model.add_to_graph(
        Triple::new(
            subject.clone(),
            NamedNode::new("https://example.com/name")?,
            Literal::new_simple_literal("Alice"),
        ),
        GraphName::NamedNode(graph),
    )?;

    let matches = model
        .find(StatementPattern {
            subject: Some(subject.as_ref().into()),
            ..StatementPattern::default()
        })
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(matches.len(), 1);
    Ok(())
}
```

`Model::find` returns a streaming `StatementMatches` iterator over a store
snapshot (ADR-005). Run `cargo run --example contexts`.

## Parsing and serialization

```rust
use oxiland::io::{Parser, Serializer, Syntax};
use oxiland::Model;

fn main() -> oxiland::Result<()> {
    let model = Model::new()?;
    Parser::for_syntax(Syntax::Turtle)
        .base_iri("https://example.com/")?
        .load_collecting(&model, b"<alice> <name> \"Alice\" .".as_slice())?;

    let ntriples = Serializer::for_syntax(Syntax::NTriples)
        .serialize_model_to_string(&model)?;
    assert!(ntriples.contains("Alice"));
    Ok(())
}
```

`Parser::load_into` inserts progressively and may leave partial data on parse
failure (ADR-007). Prefer `load_collecting` when you need parse-then-insert
batching without transactions. See [I/O guide](docs/users/io.md) and
`cargo run --example progressive_load`.

## Persistent storage

`Model::open` stores quads in a [Fjall](https://github.com/fjall-rs/fjall)
keyspace and keeps an Oxigraph in-memory working set for querying:

```rust,no_run
use oxiland::Model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::open("./data/oxiland-store")?;
    assert!(model.is_empty()?);
    Ok(())
}
```

**Supported (format v1):** on-disk compatibility is promised for Oxiland **0.4.x**
and **0.5.x** patch releases (ADR-006). Pre-0.4 experimental directories need
`Model::migrate_legacy_store`. Prefer N-Quads export for archival copies across
majors. Details: [persistence guide](docs/users/persistence.md).

## Documentation

Published guides: [oxiland.readthedocs.io](https://oxiland.readthedocs.io/).
API reference: [docs.rs/oxiland](https://docs.rs/oxiland).

| Audience | Start here |
|---|---|
| **Users** | [Getting started](docs/users/getting-started.md) · [docs hub](docs/index.md) |
| **Evaluators** | [Positioning](docs/evaluators/positioning.md) · [Parity ledger](PARITY.md) |
| **Contributors** | [CONTRIBUTING.md](CONTRIBUTING.md) · [Planning docs](docs/index.md#contributors) |

Also: [Changelog](CHANGELOG.md) · [FAQ](docs/users/faq.md) ·
[Security policy](SECURITY.md) · [Code of conduct](CODE_OF_CONDUCT.md)

## Architecture (summary)

```text
Rust application
      │
      ▼
Oxiland safe facade ──> Oxigraph RDF, storage, I/O, and SPARQL
      ▲
      │
Future oxiland (PyPI, 0.7+) and oxiland-capi (0.8+)
```

Roadmap highlights: 0.6 safe-API parity, 0.7 Python package, 0.8 C ABI
preview. Full plan: [docs/ROADMAP.md](docs/ROADMAP.md).

## Development

```console
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps
python3 scripts/check-inventory.py
python3 scripts/check-docs.py
scripts/generate-public-api.sh check
```

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at
your option.
