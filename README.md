# Oxiland

Oxiland is a safe Rust implementation of the Redland `librdf` object model,
using [Oxigraph](https://oxigraph.org/) as its RDF and SPARQL engine.

```rust
use oxiland::terms::{Literal, NamedNode, Triple};
use oxiland::{Model, Query, QueryResults};

let model = Model::new()?;
model.add(Triple::new(
    NamedNode::new("https://example.com/alice")?,
    NamedNode::new("https://example.com/name")?,
    Literal::new_simple_literal("Alice"),
))?;

assert!(matches!(
    Query::new("ASK { ?s ?p ?o }").execute(&model)?,
    QueryResults::Boolean(true)
));
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Compatibility target

The target is behavioral parity with Redland 1.0.17/1.0.18 documentation.
Rust callers get ownership-safe equivalents rather than C allocation functions.
A C ABI compatibility crate is planned separately so safe API design is not
coupled to raw pointers and legacy symbol names.

See [PARITY.md](PARITY.md) for the auditable implementation ledger and
[docs/README.md](docs/README.md) for the architecture, compatibility,
verification, and phased 0.x plans.
