# Getting started

This guide gets you from install to three common workflows: load statements,
query with SPARQL, and parse/serialize RDF.

## Install the toolchain

Oxiland requires **Rust 1.87+** (edition 2024).

```console
rustup update stable
rustc --version
```

If your organization pins an older toolchain, you cannot evaluate Oxiland until
that pin moves; this is intentional for the Oxigraph 0.5.9 compatibility matrix.

## Add the dependency

```toml
[dependencies]
oxiland = "0.4.0"
```

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

See [SPARQL](sparql.md) and `cargo run --example select`.

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

## What to read next

- Named graphs and matching: README “Contexts” or `cargo run --example contexts`
- SPARQL Update / results: [sparql.md](sparql.md) (`construct`, `update` examples)
- I/O details and progressive load: [io.md](io.md)
- Persistence caveats: [persistence.md](persistence.md)
- Common failures: [faq.md](faq.md)
