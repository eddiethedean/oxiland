# SPARQL queries, updates, and results

Oxiland 0.3 provides Redland-shaped [`Query`](https://docs.rs/oxiland/latest/oxiland/struct.Query.html)
and [`Update`](https://docs.rs/oxiland/latest/oxiland/struct.Update.html)
builders over Oxigraph 0.5.9 (ADR-009–ADR-012).

## ASK and SELECT

```rust
use oxiland::{Model, Query, QueryResults};
use oxiland::terms::{self, Literal, Triple};

# fn main() -> oxiland::Result<()> {
let model = Model::new()?;
model.add(Triple::new(
    terms::named_node("https://example.com/alice")?,
    terms::named_node("https://example.com/name")?,
    Literal::new_simple_literal("Alice"),
))?;

assert!(matches!(
    Query::new("ASK { ?s ?p ?o }").execute(&model)?,
    QueryResults::Boolean(true)
));

let results = Query::new(
    "SELECT ?name WHERE { <https://example.com/alice> <https://example.com/name> ?name }",
)
.limit(10)?
.execute(&model)?;

if let QueryResults::Solutions(mut solutions) = results {
    let solution = solutions.next().expect("one row")
        .map_err(|e| oxiland::Error::SparqlEvaluation(e.to_string()))?;
    // Unbound variables are None (name or position).
    assert!(solution.get("name").is_some());
    assert!(solution.get("missing").is_none());
}
# Ok(())
# }
```

## CONSTRUCT / DESCRIBE

Graph results stream as `QueryResults::Graph`. Serialize them with
[`serialize_graph_results_to_writer`](https://docs.rs/oxiland/latest/oxiland/fn.serialize_graph_results_to_writer.html)
and the 0.2 RDF
[`Serializer`](https://docs.rs/oxiland/latest/oxiland/io/struct.Serializer.html),
not SPARQL Results formats.

Runnable: `cargo run --example construct`.

## Update

```rust
use oxiland::{Model, Update};

# fn main() -> oxiland::Result<()> {
let model = Model::new()?;
Update::new(
    "INSERT DATA { <https://example.com/a> <https://example.com/p> \"x\" }",
)
.execute(&model)?;
# Ok(())
# }
```

Fjall-backed models (`Model::open`) resync durable storage after a successful
update under the write lock; sync failure restores the pre-update on-disk key
set and rolls memory back to that snapshot. Dataset builders on Update require
operations that expose USING datasets (typically `DELETE/INSERT`); `INSERT DATA`,
`DELETE DATA`, and similar ops return `Unsupported`.
Runnable: `cargo run --example update`.

## Configuration

| Builder method | Effect |
|---|---|
| `base_iri` / `prefix` | Parsing defaults (`InvalidRdf` on bad IRIs) |
| `limit` / `offset` | Algebra `Slice` for SELECT/CONSTRUCT/DESCRIBE (not ASK; replaces in-query limits) |
| `default_graph` / `default_graph_as_union` | Dataset selection (Query and Update). An empty `default_graph([])` list selects an empty default dataset. |
| `available_named_graphs` | Query-only named-graph availability |
| `cancellation_token` | Cooperative cancel (ADR-012); wall-clock timeout is caller-driven |

## Result serialization

[`ResultsFormat`](https://docs.rs/oxiland/latest/oxiland/enum.ResultsFormat.html)
covers XML, JSON, CSV, and TSV for ASK/SELECT via
`serialize_query_results_to_string` / `serialize_query_results_to_writer`.
Graph results use `serialize_graph_results_to_writer`.

## Escape hatch

`oxiland::sparql` re-exports Oxigraph primitives for advanced use. It is **not**
the verified compatibility surface.
