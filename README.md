# Oxiland

Oxiland is a safe Rust reimplementation of the Redland `librdf` object model,
powered by [Oxigraph](https://oxigraph.org/).

The project is working toward measurable Redland compatibility without carrying
manual C memory management into its Rust API. Oxigraph provides the RDF and
SPARQL engine; Oxiland supplies Redland-oriented concepts, behavior, migration
paths, and—later in the 0.x series—a separately audited C compatibility layer.

> [!IMPORTANT]
> Oxiland is currently an early 0.1 implementation. The core model works, but
> the crate does **not** yet provide complete Redland API, behavioral, source,
> or ABI compatibility. See the [parity ledger](PARITY.md) for verified status.

## Why Oxiland?

- Familiar Redland concepts with Rust ownership and error handling.
- Oxigraph-backed RDF terms, datasets, parsing primitives, and SPARQL.
- In-memory operation by default, with optional RocksDB persistence.
- No `unsafe` code in the primary crate.
- Compatibility claims backed by an API inventory and differential tests
  rather than an unqualified percentage.
- A staged path toward existing C consumer support.

## Current capabilities

| Capability | Status |
|---|---|
| RDF named nodes, blank nodes, literals, triples, and quads | Available through Oxigraph types |
| In-memory model | Available |
| Default-graph CRUD | Available |
| Named-graph/context insertion and matching | Partial |
| Partial statement matching | Available; currently eager |
| SPARQL query execution | Basic support |
| RDF parser and serializer primitives | Re-exported; Redland-style facade planned for 0.2 |
| Persistent RocksDB model | Optional `rocksdb` feature |
| SPARQL Update and complete result adapters | Planned for 0.3 |
| Full safe Rust Redland accounting | Planned for 0.6 |
| C source and ABI compatibility | Planned for 0.7–0.9 |

“Available” means the current public workflow is implemented and tested. It
does not imply full subsystem parity with Redland.

## Requirements

- Rust 1.87 or newer
- A supported native toolchain when enabling RocksDB

Oxiland pins Oxigraph 0.5.9 for the current release so compatibility testing is
performed against a known engine version.

## Installation

Until Oxiland is published, add it as a local or Git dependency:

```toml
[dependencies]
oxiland = { path = "../oxiland" }
```

The default build uses Oxigraph's in-memory backend and does not compile
RocksDB. Enable persistent storage explicitly:

```toml
[dependencies]
oxiland = { path = "../oxiland", features = ["rocksdb"] }
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

Run this example from the repository with:

```console
cargo run --example quick_start
```

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

    let matches = model.find(StatementPattern {
        subject: Some(subject.as_ref().into()),
        ..StatementPattern::default()
    })?;

    assert_eq!(matches.len(), 1);
    Ok(())
}
```

`Model::find` currently collects matching quads into a `Vec`. A lazy streaming
replacement is a 0.1/0.5 design priority and is tracked in the
[execution plan](docs/EXECUTION.md).

The complete example is runnable with `cargo run --example contexts`.

## Persistent storage

With the `rocksdb` feature enabled:

```rust,no_run
use oxiland::Model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::open("./data/oxiland")?;
    assert!(model.is_empty()?);
    Ok(())
}
```

On-disk compatibility and migration guarantees are not yet stabilized. Do not
treat 0.x persistent stores as archival formats without an independent export.

## Relationship to Redland

Oxiland targets the public `librdf` 1.0.17 API and the reference manual labeled
1.0.18. Raptor and Rasqal behavior is included only when exposed through a
public `librdf` workflow.

Compatibility is reported at separate levels:

1. **Concept parity** — Redland workflows have Rust equivalents.
2. **Safe API accounting** — every public Redland item is mapped or classified.
3. **Behavioral parity** — equivalent operations match native Redland fixtures.
4. **C source compatibility** — supported C programs compile against Oxiland.
5. **C ABI compatibility** — supported existing binaries can load the
   compatibility library.
6. **Downstream compatibility** — selected real consumers pass unchanged.

This distinction matters: safe Rust may replace allocation functions
idiomatically while the future C layer must still reproduce their observable
ownership behavior. Details live in the
[compatibility plan](docs/COMPATIBILITY.md).

## Architecture

The primary `oxiland` crate remains safe and delegates standards-heavy work to
Oxigraph:

```text
Rust application
      │
      ▼
Oxiland safe facade ──> Oxigraph RDF, storage, I/O, and SPARQL
      ▲
      │
Future oxiland-capi
      ▲
      │
Existing C application
```

The future `oxiland-capi` crate will isolate opaque handles, strings,
allocators, callbacks, panic containment, and other audited `unsafe` code. The
safe crate has `#![forbid(unsafe_code)]`.

See the [architecture plan](docs/ARCHITECTURE.md) and
[decision log](docs/DECISIONS.md) for the full boundaries and open decisions.

## Roadmap

| Release | Intended outcome |
|---|---|
| 0.1 | Core terms, models, contexts, and basic queries |
| 0.2 | Redland-shaped RDF parsers and serializers |
| 0.3 | Complete query, results, and update workflows |
| 0.4 | Durable storage, transactions, and backend capabilities |
| 0.5 | Streams, utilities, logging, and observability |
| 0.6 | Fully accounted safe Rust Redland surface |
| 0.7 | C ABI preview |
| 0.8 | C and downstream ecosystem validation |
| 0.9 | API/ABI-frozen 1.0 release candidate |

Milestones are evidence-gated rather than date-gated. The detailed
[0.x roadmap](docs/ROADMAP.md) defines deliverables, dependencies, exclusions,
and release criteria.

## Project documentation

- [Planning index](docs/README.md)
- [Parity ledger](PARITY.md)
- [0.x roadmap](docs/ROADMAP.md)
- [Execution plan and current backlog](docs/EXECUTION.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Compatibility contract](docs/COMPATIBILITY.md)
- [Verification and release gates](docs/VERIFICATION.md)
- [Architecture decisions](docs/DECISIONS.md)
- [Risk register](docs/RISKS.md)

The parity ledger describes what exists now. Planning documents describe the
intended path and must not be read as implemented functionality.

## Development

Run the default local checks:

```console
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps
```

Also test feature-minimal and RocksDB builds when changing storage or feature
behavior:

```console
cargo test --no-default-features
cargo test --all-features
```

Compatibility work should be implemented as a vertical slice: inventory
mapping, public API, implementation, positive and failure tests, differential
evidence where applicable, and parity-ledger updates. The
[execution plan](docs/EXECUTION.md) defines readiness and completion.

## Contributing

Contributions are welcome while the API is evolving. Before proposing a broad
facade, storage, or FFI change:

1. identify the affected Redland subsystem or inventory entries;
2. read the relevant architecture and compatibility decisions;
3. define observable behavior and unsupported cases;
4. include tests through the public Oxiland API;
5. update the parity ledger and planning evidence.

The most valuable current tasks are listed in the
[0.1 backlog](docs/EXECUTION.md#current-01-backlog).

## License

Oxiland is licensed under either of:

- Apache License, Version 2.0
- MIT License

at your option.
