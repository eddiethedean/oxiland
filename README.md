# Oxiland

[![CI](https://github.com/eddiethedean/oxiland/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/oxiland/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/oxiland.svg)](https://crates.io/crates/oxiland)
[![API docs (docs.rs)](https://img.shields.io/docsrs/oxiland?label=API%20docs)](https://docs.rs/oxiland)
[![Guides (Read the Docs)](https://img.shields.io/readthedocs/oxiland?label=Guides)](https://oxiland.readthedocs.io/en/latest/)
[![MSRV](https://img.shields.io/crates/msrv/oxiland)](https://crates.io/crates/oxiland)
[![License](https://img.shields.io/crates/l/oxiland)](LICENSE-APACHE)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/eddiethedean/oxiland)

**Oxiland** gives you safe Rust (and Python) APIs for RDF models, named graphs,
SPARQL, and streaming Turtle / N-Triples / N-Quads / TriG / RDF/XML—on pinned
[Oxigraph](https://oxigraph.org/) 0.5.9, with `#![forbid(unsafe_code)]`.

Choose it when you want Redland-*shaped* workflows and explicit unsupported
errors. Prefer Oxigraph alone for greenfield engine-only apps. Oxiland is **not**
a C/ABI drop-in (planned for 0.8+).

Version **0.7.0** includes in-memory and durable Fjall storage (format v1),
`oxiland-cli` rdfproc-shaped workflows, utilities/logging, and
`pip install oxiland`. Compatibility claims are evidence-scoped in the
[parity ledger](PARITY.md)—not “100% Redland drop-in.”

> [!IMPORTANT]
> “Redland-shaped” means familiar concepts and inventories, not C or ABI
> compatibility. See the [parity ledger](PARITY.md) and
> [0.7 report](docs/reports/0.7.md).

## When to use Oxiland

| Choose Oxiland when… | Prefer Oxigraph directly when… |
|---|---|
| You want Redland-like models, contexts, and I/O facades | You only need Oxigraph’s store/SPARQL/I/O API |
| You need explicit `Unsupported` / typed errors for missing features | You want every Oxigraph capability without a facade |
| You are migrating Redland *concepts* into safe Rust or Python | You are starting a greenfield Oxigraph app |
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
| Persistent Fjall model | Available; format v1 via `Model::open` / `open_with` |
| Transactions / sync / clear | Available; `Model::transaction`, `sync`, `clear` |
| SPARQL Update and results serialization | Available; XML/JSON/CSV/TSV + graph serialize helper |
| Digests / URI / Unicode / vocab helpers | Available; `oxiland::utility` |
| World logging | Available; handlers + optional `tracing` feature |
| Python package (Pythonic PyPI API) | Available; `pip install oxiland` |
| Header-derived safe-API accounting | Available for the 0.6 `librdf` inventory (classification, not drop-in parity) |
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

| Component | Requirement |
|---|---|
| Rust (crate + CLI) | **1.87+** (edition 2024; Oxigraph 0.5.9 pin) |
| Python (PyPI) | CPython **3.10–3.13**; published **wheels only** (no sdist) |
| Optional Cargo feature | `tracing` — bridges World logging to the `tracing` crate |
| CLI | separate crate: `cargo install oxiland-cli` |

Install or update Rust with [rustup](https://rustup.rs/):

```console
rustup update stable
rustc --version   # >= 1.87
```

## Installation

**Rust**

```toml
[dependencies]
oxiland = "0.7.0"
# optional:
# oxiland = { version = "0.7.0", features = ["tracing"] }
```

**Python**

```console
pip install oxiland
```

**CLI**

```console
cargo install oxiland-cli
```

## Quick start (Rust)

```console
cargo new hello-oxiland && cd hello-oxiland
```

Add the dependency above, then:

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

From a checkout: `cargo run --example quick_start`.

## Quick start (Python)

```console
pip install oxiland
```

```python
from oxiland import Literal, Model, NamedNode, Triple, query

model = Model()
model.add(
    Triple(
        NamedNode("https://example.com/alice"),
        NamedNode("https://example.com/name"),
        Literal("Alice"),
    )
)
assert query(model, "ASK { ?s ?p ?o }") is True
```

More: [Getting started](docs/users/getting-started.md) ·
[Python guide](docs/users/python.md) · [Examples index](docs/users/examples.md).

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
snapshot. Run `cargo run --example contexts`.

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
failure. Prefer `load_collecting` when you need parse-then-insert batching
without transactions. See [I/O guide](docs/users/io.md) and
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

**Supported (format v1):** on-disk compatibility is promised for Oxiland
**0.4.x–0.7.x** patch releases. Pre-0.4 experimental directories need
`Model::migrate_legacy_store`. Prefer N-Quads export for archival copies across
majors. Details: [persistence guide](docs/users/persistence.md).

## Documentation

Published guides: [oxiland.readthedocs.io](https://oxiland.readthedocs.io/).
API reference: [docs.rs/oxiland](https://docs.rs/oxiland) (Rust) ·
[Python API landing](docs/users/python-api.md).

| Audience | Start here |
|---|---|
| **Users** | [Getting started](docs/users/getting-started.md) · [docs hub](docs/index.md) |
| **Evaluators** | [Positioning](docs/evaluators/positioning.md) · [Parity ledger](PARITY.md) |
| **Contributors** | [CONTRIBUTING.md](CONTRIBUTING.md) · [Planning docs](docs/index.md#contributors) |

Also: [Changelog](CHANGELOG.md) · [FAQ](docs/users/faq.md) ·
[Support](SUPPORT.md) · [Security policy](SECURITY.md) ·
[Code of conduct](CODE_OF_CONDUCT.md)

## Architecture (summary)

```text
Rust / Python application
      │
      ▼
Oxiland safe facade ──> Oxigraph RDF, storage, I/O, and SPARQL
```

Roadmap highlights: 0.8 C ABI preview, 0.9 downstream C validation. Full plan:
[docs/ROADMAP.md](docs/ROADMAP.md).

## Development

```console
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps
python3 scripts/check-inventory.py
python3 scripts/check-docs.py
scripts/generate-public-api.sh check
cd python && maturin develop && pytest && pyright
```

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at
your option.
