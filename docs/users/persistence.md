# Persistence

`Model::new()` creates an in-memory store. `Model::open(path)` /
`Model::open_with(OpenOptions::fjall(path))` opens a **supported** Fjall-backed
durable store (Oxiland format v1, ADR-006) plus an Oxigraph working set.

## Stability (ADR-006)

- Format v1 stores `__oxiland/meta` beside N-Quads keys. **0.4.x** and **0.5.x**
  open format v1 without migration.
- Pre-0.4 experimental directories (no metadata) must call
  `Model::migrate_legacy_store` before `open`.
- Prefer standards RDF for archival continuity across major upgrades.

## Transactions

```rust
# use oxiland::terms::{self, Literal, Triple};
# use oxiland::Model;
# fn main() -> oxiland::Result<()> {
let model = Model::new()?;
model.transaction(|tx| {
    tx.add(Triple::new(
        terms::named_node("https://example.com/alice")?,
        terms::named_node("https://example.com/name")?,
        Literal::new_simple_literal("Alice"),
    ))?;
    Ok(())
})?;
# Ok(())
# }
```

On Fjall models, durability syncs when the transaction commits. Errors and drops
roll back the Oxigraph working set without writing a new durable snapshot.

## Atomic import

Prefer `Parser::load_transactional` (or `Model::import_nquads_from_path`) when a
mid-parse failure must not leave durable partial data. Progressive `load_into`
still syncs each successful insert. Import merges quads into the existing model
(RDF union); clear first if you need a replace restore.

## Read-only and capabilities

```rust
# use oxiland::{Model, OpenOptions};
# let path = std::env::temp_dir().join("oxiland-doc-ro");
# let _ = Model::open(&path);
let model = Model::open_with(OpenOptions::fjall(&path).read_only(true))?;
assert!(model.capabilities().read_only);
# Ok::<(), oxiland::Error>(())
```

## Export before major upgrades

```rust
use oxiland::Model;
# let model = Model::new()?;

model.export_nquads_to_path("backup.nq")?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Use TriG via `Serializer` when you need a compact named-graph archive.

## See also

- [FAQ](faq.md)
- [Storage API design](../design/0.4-storage-api.md)
- [Legacy backend disposition](../design/0.4-legacy-storage.md)
- [Roadmap](../ROADMAP.md)
