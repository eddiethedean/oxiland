# SPARQL

Oxiland 0.2 exposes basic query execution through [`Query`](https://docs.rs/oxiland/latest/oxiland/struct.Query.html).
The query text is stored until [`Query::execute`](https://docs.rs/oxiland/latest/oxiland/struct.Query.html#method.execute);
parse failures become `Error::SparqlParse`, evaluation failures
`Error::SparqlEvaluation`.

Limit, offset, SPARQL Update, richer result adapters, and cancellation policy
are planned for **0.3**.

## ASK

```rust
use oxiland::{Model, Query, QueryResults};
# use oxiland::terms::{Literal, NamedNode, Triple};
# let model = Model::new()?;
# model.add(Triple::new(
#     NamedNode::new("https://example.com/alice")?,
#     NamedNode::new("https://example.com/name")?,
#     Literal::new_simple_literal("Alice"),
# ))?;

let results = Query::new("ASK { ?s <https://example.com/name> ?o }").execute(&model)?;
assert!(matches!(results, QueryResults::Boolean(true)));
# Ok::<(), oxiland::Error>(())
```

## SELECT

```rust
use oxiland::terms::{self, Literal, Triple};
use oxiland::{Model, Query, QueryResults};

fn main() -> oxiland::Result<()> {
    let model = Model::new()?;
    model.add(Triple::new(
        terms::named_node("https://example.com/alice")?,
        terms::named_node("https://example.com/name")?,
        Literal::new_simple_literal("Alice"),
    ))?;

    let results = Query::new(
        "SELECT ?name WHERE { <https://example.com/alice> <https://example.com/name> ?name }",
    )
    .execute(&model)?;

    match results {
        QueryResults::Solutions(solutions) => {
            let mut found = false;
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    oxiland::Error::SparqlEvaluation(e.to_string())
                })?;
                if let Some(term) = solution.get("name") {
                    found = true;
                    assert_eq!(term.to_string(), "\"Alice\"");
                }
            }
            assert!(found);
        }
        other => panic!("expected solutions, got another QueryResults variant"),
    }
    Ok(())
}
```

Runnable: `cargo run --example select`.

## What is not wrapped yet

- CONSTRUCT / DESCRIBE convenience helpers (engine may still accept query text;
  result adapters are 0.3)
- SPARQL Update
- Query dataset / default-graph overrides on the Oxiland facade
- Guaranteed result ordering beyond SPARQL’s own rules

For engine-level control, `oxiland::sparql` re-exports Oxigraph primitives, but
those are advanced escape hatches—not the compatibility surface.
