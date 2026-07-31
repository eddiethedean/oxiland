# Rust streams and iterators

Oxiland uses standard Rust iterators for statement matching, RDF parsing, and
SPARQL results. The concrete iterator types differ, but they follow one resource
and failure policy.

## Contract

1. Items are produced lazily; collection is always caller-controlled.
2. Fallible streams yield `Result<Item>` so storage, parsing, or evaluation
   failures can appear after successful items.
3. `break` or dropping the iterator stops the operation and releases its owned
   state through RAII.
4. Iteration order is not a general dataset guarantee unless a SPARQL `ORDER BY`
   or another documented API contract defines it.
5. A stream is not automatically a snapshot copy; it owns or borrows the state
   documented by its concrete API.

## Surfaces

| Workflow | Entry point | Item |
|---|---|---|
| Statement matching | `Model::find` | `Result<Quad>` |
| Reader RDF parse | `Parser::parse_reader` / `parse_path` | `Result<Quad>` |
| Slice RDF parse | `Parser::parse_slice` / `parse_str` | `Result<Quad>` |
| SPARQL SELECT | `QueryResults::Solutions` | Oxigraph solution result |
| SPARQL CONSTRUCT / DESCRIBE | `QueryResults::Graph` | Oxigraph triple result |

## Process incrementally

```rust
use oxiland::terms::{Literal, Triple, named_node};
use oxiland::{Model, StatementPattern};

# fn main() -> oxiland::Result<()> {
let model = Model::new()?;
model.add(Triple::new(
    named_node("https://example.com/s")?,
    named_node("https://example.com/p")?,
    Literal::new_simple_literal("value"),
))?;

for item in model.find(StatementPattern::default()) {
    let quad = item?;
    println!("{quad}");
    break;
}
# Ok(())
# }
```

Avoid `collect::<Result<Vec<_>, _>>()` for unbounded or caller-controlled
results unless the application has already enforced a size limit.

## Error timing

Parser configuration can succeed even when a later record is malformed. Query
execution can return a result iterator whose evaluation later fails. Handle
errors at the item boundary:

```rust,no_run
# use oxiland::io::{Parser, Syntax};
let stream = Parser::for_syntax(Syntax::Turtle).parse_path("incoming.ttl")?;
for item in stream {
    match item {
        Ok(quad) => process(quad),
        Err(error) => {
            report(error);
            break;
        }
    }
}
# fn process(_: oxiland::terms::Quad) {}
# fn report(_: oxiland::Error) {}
# Ok::<(), oxiland::Error>(())
```

For progressive model loading, successful items before a failure may already be
durable. Iterator-level laziness does not imply transactional rollback.

## Ownership and cancellation

`StatementMatches` yields owned quads from store state and does not borrow the
`Model`. Parser streams own their reader or borrow their input slice according
to the method signature. SPARQL result adapters remain tied to query/model state
through their Rust lifetimes.

SPARQL supports cooperative `CancellationToken` handling. Parse and bulk-load
iterators do not expose wall-clock cancellation; stop consuming them or apply
process/thread isolation at the application boundary.

## Production practices

- Apply SPARQL limits and application-level budgets before exposing a stream.
- Keep expensive iterators out of long-lived unrelated waits.
- Track processed count, duration, early termination, and concrete error class.
- Use streaming writers to avoid materializing serialized output as a `String`.
- Do not assume streaming bounds the model's own memory footprint; persistent
  models keep an in-memory working set.

See [RDF input and output](io.md), [SPARQL](sparql.md), and
[Rust production operations](rust-production.md).
