# Persistence

`Model::new()` creates an in-memory store. `Model::open(path)` opens an
**experimental** Fjall-backed durable copy plus an Oxigraph working set.

## Stability

- On-disk format compatibility across Oxiland / Oxigraph versions is **not**
  promised in 0.x (see ADR-006, still proposed for 0.4).
- Do not treat 0.x Fjall directories as archival storage.
- A supported storage contract is planned for **0.4**.

## Export before upgrade

Serialize to a standards format before moving stores:

```rust
use oxiland::io::{Serializer, Syntax};
# use oxiland::Model;
# let model = Model::new()?;

let nq = Serializer::for_syntax(Syntax::NQuads).serialize_model_to_string(&model)?;
std::fs::write("backup.nq", nq)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Use N-Quads or TriG when you need named graphs.

## Progressive load on disk

`Parser::load_into` syncs each successful insert on Fjall models. A mid-parse
failure can leave durable partial data. Prefer `load_collecting` when that is
unacceptable, or load into memory and export.

## See also

- [FAQ](faq.md)
- [Roadmap 0.4](../ROADMAP.md)
