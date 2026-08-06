# Rust getting started

This guide gets Rust users from install to common workflows. Python users have
a complete, independent [Python documentation track](python.md).

## Install the toolchain

Oxiland requires **Rust 1.87+** (edition 2024).

```console
rustup update stable
rustc --version   # >= 1.87
```

If your organization pins an older toolchain, you cannot evaluate the Rust crate
until that pin moves; this is intentional for the Oxigraph 0.5.9 compatibility
matrix. The [Python package](python.md) only needs CPython 3.10–3.14.

For a Rust-only application, no system Oxigraph or Redland library is required.

## Create a project and add the dependency

```console
cargo new hello-oxiland
cd hello-oxiland
```

Tip **0.13.0** is the current package version.

**Published / tip pin:**

```toml
[dependencies]
oxiland = "0.13.0"
```

**Tip (git or path):**

```toml
[dependencies]
oxiland = { git = "https://github.com/eddiethedean/oxiland" }
```

Optional: enable `features = ["tracing"]` to bridge World logging to the
`tracing` crate. Install the CLI separately with `cargo install oxiland-cli`
(see the [CLI guide](cli.md)).

## Workflow 1 — Build a model and ASK

```rust
use oxiland::terms::{Literal, Triple, named_node};
use oxiland::{Model, Query, QueryResults};

fn main() -> oxiland::Result<()> {
    let model = Model::new()?;
    model.add(Triple::new(
        named_node("https://example.com/alice")?,
        named_node("https://example.com/name")?,
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
use oxiland::terms::{Literal, Triple, named_node};
use oxiland::{Model, Query, QueryResults};

fn main() -> oxiland::Result<()> {
    let model = Model::new()?;
    model.add(Triple::new(
        named_node("https://example.com/alice")?,
        named_node("https://example.com/name")?,
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

## Choose storage deliberately

`Model::new()` is process-local and disappears when the model is dropped.
`Model::open(path)` creates or opens a persistent local store. For production,
prefer typed `OpenOptions` with `create(false)` so a missing configured store
fails instead of initializing an empty dataset.

See [Persistence](persistence.md) and
[Rust production operations](rust-production.md) before deploying durable
state.

## Handle errors by category

Application code can use `oxiland::Result<T>` and match `Error` variants at a
boundary. Avoid matching diagnostic strings. Parse errors, SPARQL parse errors,
evaluation failures, storage failures, I/O errors, and unsupported capabilities
have separate variants.

Errors may occur while advancing lazy RDF or SPARQL iterators, so handle the
item-level `Result` rather than assuming iterator construction validates all
input.

## Python package

Prefer an isolated environment, then install with the module form of pip:

```console
python -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\Activate.ps1
python -m pip install oxiland
```

Published wheels track the latest release. Start with the
[Python overview](python.md), then use the dedicated guides for
[installation](python-installation.md), [models](python-models.md),
[RDF I/O and SPARQL](python-data.md), and
[production operations](python-production.md).

## What to read next

- [Examples index](examples.md)
- Named graphs: `cargo run --example contexts`
- SPARQL Update / results: [sparql.md](sparql.md)
- I/O and progressive load: [io.md](io.md)
- Persistence: [persistence.md](persistence.md)
- Production operations: [rust-production.md](rust-production.md)
- CLI: [cli.md](cli.md)
- Common failures: [faq.md](faq.md)
