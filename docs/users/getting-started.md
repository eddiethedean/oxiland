# Getting started

This guide gets you from install to common workflows in Rust, then points to the
Python track.

## Install the toolchain

Oxiland requires **Rust 1.87+** (edition 2024).

```console
rustup update stable
rustc --version   # >= 1.87
```

If your organization pins an older toolchain, you cannot evaluate the Rust crate
until that pin moves; this is intentional for the Oxigraph 0.5.9 compatibility
matrix. The [Python package](python.md) only needs CPython 3.10–3.13.

## Create a project and add the dependency

```console
cargo new hello-oxiland
cd hello-oxiland
```

```toml
[dependencies]
oxiland = "0.7.0"
```

Optional: enable `features = ["tracing"]` to bridge World logging to the
`tracing` crate. Install the CLI separately with `cargo install oxiland-cli`
([CLI guide](cli.md)).

## Workflow 1 — Build a model and ASK

```rust
use oxiland::terms::{Literal, NamedNode, Triple};
use oxiland::{Model, Query, QueryResults};

fn main() -> oxiland::Result<()> {
    let model = Model::new()?;
    model.add(Triple::new(
        NamedNode::new("https://example.com/alice")?,
        NamedNode::new("https://example.com/name")?,
        Literal::new_simple_literal("Alice"),
    ))?;

    let ask = Query::new("ASK { ?s ?p ?o }").execute(&model)?;
    assert!(matches!(ask, QueryResults::Boolean(true)));
    Ok(())
}
```

From a checkout: `cargo run --example quick_start`.

## Workflow 2 — SELECT bindings

```rust
use oxiland::terms::{Literal, NamedNode, Triple};
use oxiland::{Model, Query, QueryResults};

fn main() -> oxiland::Result<()> {
    let model = Model::new()?;
    model.add(Triple::new(
        NamedNode::new("https://example.com/alice")?,
        NamedNode::new("https://example.com/name")?,
        Literal::new_simple_literal("Alice"),
    ))?;

    let results = Query::new(
        "SELECT ?name WHERE { <https://example.com/alice> <https://example.com/name> ?name }",
    )
    .execute(&model)?;

    match results {
        QueryResults::Solutions(solutions) => {
            for solution in solutions {
                let solution = solution
                    .map_err(|error| oxiland::Error::SparqlEvaluation(error.to_string()))?;
                if let Some(term) = solution.get("name") {
                    println!("name = {term}");
                }
            }
        }
        _other => panic!("expected SELECT solutions"),
    }
    Ok(())
}
```

From a checkout: `cargo run --example select`. Details: [SPARQL](sparql.md).

## Workflow 3 — Parse Turtle and write N-Triples

```rust
use oxiland::io::{Parser, Serializer, Syntax};
use oxiland::Model;

fn main() -> oxiland::Result<()> {
    let model = Model::new()?;
    Parser::for_syntax(Syntax::Turtle)
        .base_iri("https://example.com/")?
        .load_collecting(&model, b"<alice> <name> \"Alice\" .".as_slice())?;

    let out = Serializer::for_syntax(Syntax::NTriples).serialize_model_to_string(&model)?;
    println!("{out}");
    Ok(())
}
```

From a checkout: `cargo run --example parse_serialize`.

## Python track

```console
pip install oxiland
```

See the [Python guide](python.md) and
[python/examples/](https://github.com/eddiethedean/oxiland/tree/main/python/examples).

## What to read next

- [Examples index](examples.md)
- Named graphs: `cargo run --example contexts`
- SPARQL Update / results: [sparql.md](sparql.md)
- I/O and progressive load: [io.md](io.md)
- Persistence: [persistence.md](persistence.md)
- CLI: [cli.md](cli.md)
- Common failures: [faq.md](faq.md)
