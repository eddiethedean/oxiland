# Oxiland

[![CI](https://github.com/eddiethedean/oxiland/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/oxiland/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/oxiland.svg)](https://crates.io/crates/oxiland)
[![PyPI](https://img.shields.io/pypi/v/oxiland?cacheSeconds=3600)](https://pypi.org/project/oxiland/)
[![API docs](https://img.shields.io/docsrs/oxiland?label=Rust%20API)](https://docs.rs/oxiland)
[![Guides](https://img.shields.io/readthedocs/oxiland?label=Guides)](https://oxiland.readthedocs.io/en/latest/)
[![MSRV](https://img.shields.io/crates/msrv/oxiland)](https://crates.io/crates/oxiland)
[![License](https://img.shields.io/crates/l/oxiland)](LICENSE-APACHE)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/eddiethedean/oxiland)

> **Release status:** tip **0.11.0** demonstrates Redland parity on the frozen
> matrix (six native differential cells, C corpus, librdf-compat packaging).
> Reproduce with `scripts/check-0.11-release.py` against
> `compatibility/qualification/`.

Oxiland is an embedded RDF toolkit for Rust and Python. It provides validated
RDF terms, in-memory and persistent datasets, named graphs, SPARQL 1.1, and
streaming RDF input and output through a compact, typed API—without running a
database server.

Tip **0.11.0** closes the demonstrated Redland-parity gate defined in the
[0.11 milestone](docs/milestones/0.11.md). See the
[parity report](docs/reports/0.11.md) and [parity ledger](PARITY.md).

| Surface | Install | Best for |
|---|---|---|
| Rust library | `oxiland = "0.11.0"` from crates.io, or path/git for tip | Native applications and services |
| Python package | `python -m pip install oxiland` | Python data pipelines and embedded RDF applications |
| Command line | `cargo install oxiland-cli` | Store inspection, imports, exports, and scripted queries |
| C ABI | Build from this repo: `cargo build -p oxiland-capi` | Redland-shaped C source + librdf-compat packaging (not on crates.io) |

Oxiland uses [Oxigraph](https://oxigraph.org/) 0.5.9 for standards-oriented RDF
and SPARQL execution and Fjall for its supported durable store. The primary
Rust crate forbids unsafe code.

## Capabilities

- RDF named nodes, blank nodes, literals, triples, quads, and graph names;
- default-graph and named-graph CRUD with lazy pattern matching;
- process-local in-memory models and local persistent format-v1 stores;
- atomic write transactions, explicit sync, read-only open, and N-Quads backup;
- SPARQL ASK, SELECT, CONSTRUCT, DESCRIBE, and Update;
- streaming Turtle, N-Triples, N-Quads, TriG, and RDF/XML parsing;
- RDF and SPARQL result serialization;
- digest, IRI, file-URI, Unicode, namespace, vocabulary, and logging utilities;
- Python wheels for CPython 3.10–3.14 with bundled type information;
- C Redland-shaped surface (`oxiland-capi`) with librdf-compat packaging;
- Redland workflow migration guidance and inventory-backed compatibility claims.

## Install

### Rust

Oxiland requires Rust **1.87 or newer**.

**Published release (crates.io):**

```toml
[dependencies]
oxiland = "0.11.0"
```

**This repository tip (0.11.0 APIs and qualification tooling):**

```toml
[dependencies]
oxiland = { git = "https://github.com/eddiethedean/oxiland" }
```

Enable `tracing` only when `World` log records should also be emitted as
`tracing` events:

```toml
[dependencies]
oxiland = { version = "0.11.0", features = ["tracing"] }
```

### Python

```console
python -m pip install oxiland
```

Released wheels support CPython 3.10–3.14 on the published platform matrix and
have no required Python dependencies. See the
[Python installation guide](docs/users/python-installation.md).

### Command line

```console
cargo install oxiland-cli
oxiland-cli --help
```

Installs the latest published CLI from crates.io.

### C ABI (tip only)

Not published to crates.io (`publish = false`). Clone this repository, then:

```console
cargo build -p oxiland-capi --release
```

See the [C ABI guide](docs/users/c-abi.md).

## Rust quick start

```rust
use oxiland::terms::{Literal, Triple, named_node};
use oxiland::{Model, Query, QueryResults};

fn main() -> oxiland::Result<()> {
    let model = Model::new()?;
    model.add(Triple::new(
        named_node("https://example.com/alice")?,
        named_node("https://schema.org/name")?,
        Literal::new_simple_literal("Alice"),
    ))?;

    let result = Query::new("ASK { ?s ?p ?o }").execute(&model)?;
    assert!(matches!(result, QueryResults::Boolean(true)));
    Ok(())
}
```

`Model::add` returns `true` when the dataset changed and `false` when the same
statement was already present.

## Python quick start

```python
from oxiland import Literal, Model, NamedNode, Triple, query

model = Model()
model.add(
    Triple(
        NamedNode("https://example.com/alice"),
        NamedNode("https://schema.org/name"),
        Literal("Alice"),
    )
)

assert query(model, "ASK { ?s ?p ?o }") is True
```

The [Python documentation track](docs/users/python.md) covers installation,
models, RDF I/O, SPARQL, production operations, and the complete public API.

## Persistence and transactions

```rust,no_run
use oxiland::terms::{Literal, Triple, named_node};
use oxiland::Model;

fn main() -> oxiland::Result<()> {
    let model = Model::open("./data/catalog")?;

    model.transaction(|tx| {
        tx.add(Triple::new(
            named_node("https://example.com/item/42")?,
            named_node("https://schema.org/name")?,
            Literal::new_simple_literal("Desk lamp"),
        ))?;
        Ok(())
    })?;

    model.sync()?;
    model.export_nquads_to_path("./backups/catalog.nq")?;
    Ok(())
}
```

Persistent stores are embedded local state, not a network database. Applications
own store-directory permissions, lifecycle, capacity, backups, and service-level
concurrency. Read the [Rust production guide](docs/users/rust-production.md)
before deploying a durable model.

## RDF formats

| Syntax | Name | Media type | Extension | Named graphs |
|---|---|---|---|---|
| Turtle | `turtle` | `text/turtle` | `.ttl` | No |
| N-Triples | `ntriples` | `application/n-triples` | `.nt` | No |
| N-Quads | `nquads` | `application/n-quads` | `.nq` | Yes |
| TriG | `trig` | `application/trig` | `.trig` | Yes |
| RDF/XML | `rdfxml` | `application/rdf+xml` | `.rdf` | No |

Unknown formats and ambiguous aliases fail with `Error::Unsupported`. Oxiland
does not silently guess syntax from document contents.

## Documentation

| Need | Start here |
|---|---|
| Python | [Overview](docs/users/python.md) · [API](docs/users/python-api.md) · [Production](docs/users/python-production.md) |
| Rust | [Overview](docs/users/rust.md) · [API on docs.rs](https://docs.rs/oxiland) · [Production](docs/users/rust-production.md) |
| Command line | [CLI guide](docs/users/cli.md) |
| C ABI | [C guide](docs/users/c-abi.md) · [Limitations](docs/users/c-abi-limitations.md) |
| Examples | [Runnable examples](docs/users/examples.md) |
| Troubleshooting | [FAQ](docs/users/faq.md) |
| Evaluation | [Positioning](docs/evaluators/positioning.md) · [Compatibility contract](docs/COMPATIBILITY.md) |
| Project | [Support](SUPPORT.md) · [Security](SECURITY.md) · [Contributing](CONTRIBUTING.md) |

Published guides are available at
[oxiland.readthedocs.io](https://oxiland.readthedocs.io/).

## Compatibility and scope

Oxiland supports Redland-shaped concepts and migration workflows. Tip **0.11.0**
ships `oxiland-capi` with demonstrated source and librdf-compat binary evidence
on the frozen matrix—see [limitations](docs/users/c-abi-limitations.md) for
remaining behavioral gaps. The Python package is not an rdflib adapter. Every
compatibility statement is scoped by subsystem, platform, and evidence in the
[parity ledger](PARITY.md).

Choose Oxigraph directly when only its native engine API is required. Choose
Oxiland when its stable facade, explicit unsupported errors, persistent-store
contract, Python package, CLI, C surface, or Redland migration evidence adds
value.

## Stability and support

Oxiland is pre-1.0. Minor 0.x releases may contain documented public API
changes. Persistent format v1 is reopen-compatible across 0.4.x–0.11.x patch
lines; export standards RDF before major upgrades. See the
[support policy](SUPPORT.md) and [changelog](CHANGELOG.md).

Milestone 0.11 demonstrated Redland parity on the frozen matrix; version 1.0
still requires the readiness gates in [ROADMAP](docs/ROADMAP.md).

Report suspected vulnerabilities privately according to
[SECURITY.md](SECURITY.md), not in a public issue.

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

Python contributors should also run the package's pytest, Pyright, examples,
and wheel checks described in [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under either [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at
your option.
