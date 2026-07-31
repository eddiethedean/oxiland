# Streams and iterators

Oxiland streams RDF and SPARQL results as standard Rust iterators.
There is **no** unifying stream trait—three shapes share one policy.

## Shared policy

1. **Lazy** — items are produced as you iterate; nothing is fully buffered unless
   you collect.
2. **Fallible items** — parser and find streams yield `Result<_>` so storage or
   syntax errors surface mid-stream.
3. **Early stop** — drop the iterator (or `break`) when you have enough results.
4. **No close callback** — RAII ends the stream; there is no Redland visitor
   callback API.

Cancellation remains **SPARQL-scoped** via `CancellationToken` (ADR-012). Parse
and bulk-load do not take a wall-clock timeout in the current API.

## Surfaces

| Workflow | Entry point | Stream type |
|---|---|---|
| Statement matching | `Model::find` | `StatementMatches` → `Result<Quad>` |
| RDF parse | `Parser::parse_reader` / `parse_str` | `QuadStream` / `SliceStream` |
| SPARQL SELECT / CONSTRUCT | `Query::execute` | `QueryResults` adapters |

## Examples

```rust
# use oxiland::{Model, StatementPattern};
# use oxiland::terms::{self, Literal, Triple};
# fn main() -> oxiland::Result<()> {
let model = Model::new()?;
model.add(Triple::new(
    terms::named_node("https://example.com/s")?,
    terms::named_node("https://example.com/p")?,
    Literal::new_simple_literal("x"),
))?;

for item in model.find(StatementPattern::default()) {
    let quad = item?;
    println!("{quad}");
    break; // early stop
}
# Ok(())
# }
```

See also [RDF I/O](io.md) and [SPARQL](sparql.md).
