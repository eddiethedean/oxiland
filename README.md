# Oxiland

Oxiland is a safe Rust reimplementation of the Redland `librdf` object model,
powered by [Oxigraph](https://oxigraph.org/).

The project is working toward measurable Redland compatibility without carrying
manual C memory management into its Rust API. Oxigraph provides the RDF and
SPARQL engine; Oxiland supplies Redland-oriented concepts, behavior, migration
paths, and—later in the 0.x series—a separately audited C compatibility layer.

> [!IMPORTANT]
> Oxiland 0.2 provides the trusted core model plus Redland-shaped RDF I/O. It
> does **not** yet provide complete Redland API accounting, full differential
> behavioral parity, source, or ABI compatibility. See the
> [parity ledger](PARITY.md) and [0.2 report](docs/reports/0.2.md).

## Why Oxiland?

- Familiar Redland concepts with Rust ownership and error handling.
- Oxigraph-backed RDF terms, datasets, parsing primitives, and SPARQL.
- In-memory operation by default, with optional [Fjall](https://github.com/fjall-rs/fjall) persistence.
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
| Named-graph/context CRUD and matching | Available |
| Partial statement matching | Available; streaming `StatementMatches` |
| SPARQL query execution | Basic ASK/SELECT support |
| RDF parser and serializer facades | Available; Turtle, N-Triples, N-Quads, TriG, RDF/XML |
| Syntax discovery by name, MIME type, and extension | Available via `Syntax` |
| Persistent Fjall model | Available via `Model::open` |
| SPARQL Update and complete result adapters | Planned for 0.3 |
| Full safe Rust Redland accounting | Planned for 0.6 |
| C source and ABI compatibility | Planned for 0.7–0.9 |

“Available” means the current public workflow is implemented and tested. It
does not imply full subsystem parity with Redland.

## Requirements

- Rust 1.87 or newer

Oxiland pins Oxigraph 0.5.9 for the current release so compatibility testing is
performed against a known engine version.

## Installation

Add Oxiland to your `Cargo.toml`:

```toml
[dependencies]
oxiland = "0.2.0"
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
snapshot (ADR-005).

The complete example is runnable with `cargo run --example contexts`.

## Parsing and serialization

Oxiland 0.2 adds Redland-shaped I/O facades. Choose a closed [`Syntax`](https://docs.rs/oxiland),
stream quads, or load into a model:

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
batching without transactions. Format lookup never silently guesses: unknown
names, ambiguous aliases such as `text/plain`/`.xml`, and the legacy `guess`
alias return `Error::Unsupported` (ADR-008).

Run `cargo run --example parse_serialize` for a complete example. Migration
notes for Redland parser/serializer callers are in
[`docs/design/0.2-io-api.md`](docs/design/0.2-io-api.md) and
[`docs/reports/0.2.md`](docs/reports/0.2.md).

## Persistent storage

`Model::open` stores quads in a [Fjall](https://github.com/fjall-rs/fjall) keyspace and keeps
an Oxigraph in-memory working set for querying:

```rust,no_run
use oxiland::Model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::open("./data/oxiland-store")?;
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
- [Project charter and 1.0 definition](docs/CHARTER.md)
- [Changelog](CHANGELOG.md)
- [Parity ledger](PARITY.md)
- [0.1 compatibility report](docs/reports/0.1.md)
- [0.2 compatibility report](docs/reports/0.2.md)
- [0.2.0 release checklist](docs/reports/0.2.0-release.md)
- [0.x roadmap](docs/ROADMAP.md)
- [Detailed 0.2 milestone plan](docs/milestones/0.2.md)
- [Execution plan and current backlog](docs/EXECUTION.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Compatibility contract](docs/COMPATIBILITY.md)
- [Verification and release gates](docs/VERIFICATION.md)
- [Architecture decisions](docs/DECISIONS.md)
- [Risk register](docs/RISKS.md)
- [0.1 inventory](compatibility/inventory/redland-1.0.17-oxiland-0.1.json)
- [0.2 inventory](compatibility/inventory/redland-1.0.17-oxiland-0.2.json)

The parity ledger describes what exists now. Planning documents describe the
intended path and must not be read as implemented functionality.

## Development

Run the default local checks:

```console
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps
python3 scripts/check-inventory.py
python3 scripts/check-docs.py
scripts/generate-public-api.sh check
```

CI runs these checks on stable and Rust 1.87.

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
[0.3 backlog](docs/EXECUTION.md#current-03-backlog).
See [CONTRIBUTING.md](CONTRIBUTING.md) for the vertical-slice and review
checklists.

## License

Oxiland is licensed under either of:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.
