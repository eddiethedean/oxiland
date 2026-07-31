# Oxiland for Rust

Oxiland is a safe Rust library for embedded RDF datasets. It combines RDF
terms, default and named graphs, streaming I/O, SPARQL, transactions, and a
supported persistent-store contract behind one error model.

## Install

Oxiland requires Rust 1.87+. crates.io currently publishes **0.8.0**; this tip
is **0.9.0** (unreleased) until the tag.

**Published release (crates.io):**

```toml
[dependencies]
oxiland = "0.8.0"
```

**This repository tip (0.9.0 APIs):**

```toml
[dependencies]
oxiland = { git = "https://github.com/eddiethedean/oxiland" }
```

Or use a local path dependency against a checkout. The default feature set has
no required native library dependency. The optional `tracing` feature forwards
`World` log records to the `tracing` ecosystem.

## First dataset

```rust
use oxiland::terms::{Literal, Triple, named_node};
use oxiland::{Model, Query, QueryResults};

fn main() -> oxiland::Result<()> {
    let model = Model::new()?;
    let statement = Triple::new(
        named_node("https://example.com/alice")?,
        named_node("https://schema.org/name")?,
        Literal::new_simple_literal("Alice"),
    );

    assert!(model.add(statement.clone())?);
    assert!(model.contains(statement.as_ref())?);

    let answer = Query::new("ASK { ?s ?p ?o }").execute(&model)?;
    assert!(matches!(answer, QueryResults::Boolean(true)));
    Ok(())
}
```

## Public surface

| Area | Primary API |
|---|---|
| RDF values | `oxiland::terms` re-exports and validated helpers |
| Dataset | `Model`, `StatementPattern`, `StatementMatches` |
| Storage | `OpenOptions`, `StorageBackend`, `StorageCapabilities` |
| Transactions | `Model::transaction`, `ModelTransaction` |
| RDF I/O | `io::Syntax`, `io::Parser`, `io::Serializer` |
| SPARQL | `Query`, `Update`, `QueryResults`, `ResultsFormat` |
| Utilities | `utility` digests, IRIs, Unicode, namespaces, vocabularies |
| Logging | `World`, `LogLevel`, `LogFacility`, optional `tracing` |
| Errors | `Error`, `ParseError`, and `Result<T>` |

The complete item-level API reference is on
[docs.rs](https://docs.rs/oxiland). These guides explain workflow choices and
operational consequences.

## Models and named graphs

`Model::new()` creates an in-memory dataset. Statements are set-like: duplicate
insertion returns `false`. Use `add_to_graph`, `remove_from_graph`, and
`StatementPattern::graph_name` for named graphs.

```rust
use oxiland::terms::{GraphName, Literal, Triple, named_node};
use oxiland::{Model, StatementPattern};

# fn main() -> oxiland::Result<()> {
let model = Model::new()?;
let graph = named_node("https://example.com/graph/people")?;
let triple = Triple::new(
    named_node("https://example.com/alice")?,
    named_node("https://schema.org/name")?,
    Literal::new_simple_literal("Alice"),
);

model.add_to_graph(triple, GraphName::NamedNode(graph.clone()))?;

for item in model.find(StatementPattern {
    graph_name: Some(GraphName::NamedNode(graph).as_ref()),
    ..StatementPattern::default()
}) {
    println!("{}", item?);
}
# Ok(())
# }
```

`Model::find()` is lazy and yields `Result<Quad>`. Process the iterator directly
for large results instead of collecting it unless the result is known to be
bounded.

## Persistent models

```rust,no_run
use oxiland::{Model, OpenOptions};

fn main() -> oxiland::Result<()> {
    let writable = Model::open("./data/catalog")?;
    let reader = Model::open_with(
        OpenOptions::fjall("./data/catalog")
            .read_only(true)
            .create(false),
    )?;
    assert!(reader.capabilities().read_only);
    writable.sync()?;
    Ok(())
}
```

For production, open existing stores with `create(false)`, use transactions for
logical writes, and maintain portable N-Quads backups. See
[Rust production operations](rust-production.md).

## Error handling

Use `oxiland::Result<T>` inside libraries and match recoverable categories at
application boundaries:

```rust,no_run
use oxiland::{Error, Model};

match Model::open("./data/catalog") {
    Ok(model) => run(model),
    Err(Error::OpenStore { path, message }) => {
        eprintln!("cannot open {}: {message}", path.display());
    }
    Err(error) => return Err(error.into()),
}
# fn run(_: Model) {}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Do not match error strings. Public variants distinguish invalid RDF, parse,
serialization, SPARQL parse/evaluation, storage, I/O, unsupported capability,
and store-open failures.

## Guide map

1. [Rust getting started](getting-started.md)
2. [RDF input and output](io.md)
3. [SPARQL queries and updates](sparql.md)
4. [Persistence](persistence.md)
5. [Streams and iterators](streams.md)
6. [Utilities and logging](utilities.md)
7. [Production operations](rust-production.md)
8. [Runnable examples](examples.md#rust-examples)

For Redland migrations, start with the
[workflow migration guide](../evaluators/migration-from-redland.md) rather than
translating C ownership patterns directly.
