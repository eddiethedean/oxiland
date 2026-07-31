# Rust SPARQL queries and updates

Oxiland provides configured `Query` and `Update` builders over its `Model`.
Queries return a result enum that distinguishes boolean, solution, and graph
results without materializing the complete response.

## ASK and SELECT

```rust
use oxiland::terms::{Literal, Triple, named_node};
use oxiland::{Model, Query, QueryResults};

# fn main() -> oxiland::Result<()> {
let model = Model::new()?;
model.add(Triple::new(
    named_node("https://example.com/alice")?,
    named_node("https://schema.org/name")?,
    Literal::new_simple_literal("Alice"),
))?;

let ask = Query::new("ASK { ?s ?p ?o }").execute(&model)?;
assert!(matches!(ask, QueryResults::Boolean(true)));

let rows = Query::new(
    "SELECT ?name WHERE { <https://example.com/alice> <https://schema.org/name> ?name }",
)
.limit(100)?
.execute(&model)?;

if let QueryResults::Solutions(solutions) = rows {
    for item in solutions {
        let solution = item
            .map_err(|error| oxiland::Error::SparqlEvaluation(error.to_string()))?;
        if let Some(name) = solution.get("name") {
            println!("{name}");
        }
    }
}
# Ok(())
# }
```

Unbound SELECT variables are absent from the solution and return `None` through
`get`. Evaluation errors may occur while advancing the lazy iterator.

## CONSTRUCT and DESCRIBE

Graph-producing queries return `QueryResults::Graph`, a lazy stream of triples:

```rust,no_run
use oxiland::{Model, Query, QueryResults};

fn run(model: &Model) -> oxiland::Result<()> {
    let results = Query::new(
        "CONSTRUCT { ?s <https://example.com/seen> ?o } WHERE { ?s ?p ?o }",
    )
    .limit(1_000)?
    .execute(model)?;

    if let QueryResults::Graph(triples) = results {
        for item in triples {
            println!("{}", item.map_err(|error|
                oxiland::Error::SparqlEvaluation(error.to_string()))?);
        }
    }
    Ok(())
}
```

Serialize graph results as RDF with `serialize_graph_results_to_writer` and an
`io::Serializer`. SPARQL Results JSON/XML/CSV/TSV formats apply only to ASK and
SELECT.

## Query configuration

| Builder | Behavior |
|---|---|
| `base_iri` | Resolve relative IRIs while parsing the query |
| `prefix` | Add a parser prefix without modifying query text |
| `limit` / `offset` | Apply an algebra slice to SELECT/CONSTRUCT/DESCRIBE |
| `default_graph` | Select one or more graphs as the query default graph |
| `default_graph_as_union` | Use the union of named graphs as the default graph |
| `available_named_graphs` | Restrict graphs addressable through `GRAPH` |
| `cancellation_token` | Attach cooperative cancellation |

API-level `limit` and `offset` replace an in-query slice and reject ASK queries.
An empty `default_graph` list selects an empty default dataset rather than the
store's normal default graph.

## Updates

```rust
use oxiland::{Model, Update};

# fn main() -> oxiland::Result<()> {
let model = Model::new()?;
Update::new(
    "INSERT DATA { <https://example.com/a> <https://example.com/p> \"value\" }",
)
.execute(&model)?;
assert_eq!(model.len()?, 1);
# Ok(())
# }
```

Persistent updates resynchronize durable state after successful execution. If
durable sync fails, Oxiland restores the pre-update disk key set and rolls the
in-memory model back to that snapshot.

Dataset configuration for updates requires operations whose SPARQL algebra
supports USING datasets. `INSERT DATA`, `DELETE DATA`, and similar forms return
`Error::Unsupported` when incompatible dataset builders are applied.

## Cooperative cancellation

```rust,no_run
use std::thread;
use std::time::Duration;
use oxiland::sparql::CancellationToken;
use oxiland::{Model, Query};

fn run(model: &Model) -> oxiland::Result<()> {
    let token = CancellationToken::new();
    let deadline = token.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        deadline.cancel();
    });

    let _ = Query::new("SELECT * WHERE { ?s ?p ?o }")
        .limit(10_000)?
        .cancellation_token(token)
        .execute(model)?;
    Ok(())
}
```

Cancellation is cooperative. The future that triggers `cancel()` defines the
wall-clock deadline; completion is not a hard real-time guarantee. Combine it
with input limits, result limits, and worker isolation for untrusted workloads.

## Result serialization

```rust,no_run
use oxiland::{Query, ResultsFormat, serialize_query_results_to_writer};
# use oxiland::Model;
# let model = Model::new()?;

let results = Query::new("SELECT ?s WHERE { ?s ?p ?o }")
    .limit(100)?
    .execute(&model)?;
serialize_query_results_to_writer(
    results,
    ResultsFormat::Json,
    std::io::stdout().lock(),
)?;
# Ok::<(), oxiland::Error>(())
```

`ResultsFormat` supports XML, JSON, CSV, and TSV. Prefer the writer API for
large result sets; the string helper buffers the serialized response.

## Error handling

| Error | Meaning |
|---|---|
| `Error::InvalidRdf` | Invalid configured base or prefix IRI |
| `Error::SparqlParse` | Query or update text could not be parsed |
| `Error::SparqlEvaluation` | Execution or result iteration failed |
| `Error::Storage` | Model read, update, rollback, or durable sync failed |
| `Error::Unsupported` | Builder/query combination is outside the public contract |

Do not match diagnostic strings. Use error variants for control flow and retain
the full message for logs or user diagnostics.

## Production guidance

- Use `ORDER BY` when output order is part of the application contract.
- Apply explicit result limits to caller-facing queries.
- Do not run unrestricted SPARQL from untrusted clients in a shared worker.
- Stream results to their destination and stop early when possible.
- Avoid logging raw query text and bound literals unless the data policy allows it.
- Instrument parse, first-result, total-result, cancellation, and error timing.

Advanced Oxigraph primitives are re-exported under `oxiland::sparql`, but they
are an escape hatch rather than the verified Oxiland compatibility surface.

See [Streams and iterators](streams.md) and
[Rust production operations](rust-production.md).
