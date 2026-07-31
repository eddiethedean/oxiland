# Rust persistence and transactions

`Model::new()` creates an in-memory dataset. `Model::open()` and
`Model::open_with()` create or open a local Fjall-backed format-v1 store and
load an in-memory RDF working set for queries.

## Open a store

```rust,no_run
use oxiland::{Model, OpenOptions};

let development = Model::open("./data/catalog")?;

let production = Model::open_with(
    OpenOptions::fjall("/srv/app-data/catalog")
        .create(false)
        .read_only(false),
)?;

let inspection = Model::open_with(
    OpenOptions::fjall("/srv/app-data/catalog")
        .create(false)
        .read_only(true),
)?;
# Ok::<(), oxiland::Error>(())
```

`Model::open(path)` allows initialization when the path is missing. Prefer
`create(false)` in production when a missing or incorrect mount must fail
instead of creating an empty dataset.

`capabilities()` reports the backend, persistence, transaction, sync, and
read-only properties. Mutating a read-only model returns `Error::Unsupported`.

## Frozen backend discovery

`supported_backends()` returns descriptors for the 1.0-intent matrix even when
an optional adapter is disabled in the current build. Each descriptor includes
the canonical name, Cargo feature, compiled state, durability, and
`LayoutReaderPolicy`. `compiled_backends()` returns only adapters usable by the
current binary. This distinction lets configuration validators recognize
`rocksdb`, for example, while still returning an explicit error when
`storage-rocksdb` was not compiled.

The supported identities are `memory`, `fjall`, `redb`, `rocksdb`, `sqlite`,
and `lmdb`. All durable adapters retain a format-v1 reader/export path;
standards RDF is the portable cross-backend format. The physical custom-backend
adapter remains sealed for 1.0 (ADR-024).

## Transaction contract

```rust,no_run
use oxiland::terms::{Literal, Triple, named_node};
use oxiland::Model;

fn activate(model: &Model) -> oxiland::Result<()> {
    model.transaction(|tx| {
        tx.add(Triple::new(
            named_node("https://example.com/alice")?,
            named_node("https://example.com/status")?,
            Literal::new_simple_literal("active"),
        ))?;
        Ok(())
    })
}
```

- Returning `Ok` commits the complete operation set.
- Returning `Err` rolls back.
- Persistent commits synchronize durable state.
- Nested transactions return `Error::Unsupported`.
- Auto-commit mutation while a transaction is open is rejected.
- Same-thread reads during the callback see the last committed working set,
  not uncommitted changes.

The transaction handle supports add, insert, remove, clear-graph, and full clear
operations. Keep all mutations for the logical write on that handle.

## Atomic imports

```rust,no_run
use oxiland::io::{GraphTarget, Parser, Syntax};
# use oxiland::Model;
# let model = Model::new()?;

Parser::for_syntax(Syntax::NQuads)
    .graph_target(GraphTarget::Dataset)
    .load_path_transactional(&model, "snapshot.nq")?;
# Ok::<(), oxiland::Error>(())
```

`load_transactional` parses the complete input and commits it in one
transaction. `load_collecting` also parses first but uses best-effort removal if
an insertion later fails. `load_into` is progressive and can leave durable
partial data. See [RDF input and output](io.md#load-into-a-model).

## Sync and durability

Persistent auto-commit writes and successful transactions update the durable
store. `Model::sync()` provides an explicit durability boundary and surfaces a
storage failure to the caller. Call it before controlled shutdown, backups, and
handoffs where the application needs an explicit acknowledgment.

An in-memory model accepts `sync()` as a no-op through the same interface.

## Backup and restore

N-Quads is the portable archival format because it preserves graph names:

```rust,no_run
use oxiland::Model;

fn backup(model: &Model) -> oxiland::Result<()> {
    model.sync()?;
    model.export_nquads_to_path("./backups/catalog.nq")
}

fn restore() -> oxiland::Result<Model> {
    let model = Model::open("./data/restored")?;
    let processed = model.import_nquads_from_path("./backups/catalog.nq")?;
    eprintln!("restored {processed} quad(s)");
    Ok(model)
}
```

Import merges with existing data and does not clear first. Restore into a new
store or deliberately clear the selected target when replacement is required.
Store backup files outside the live database directory and test restore
regularly.

## Format compatibility

Format v1 stores metadata beside durable N-Quads keys. Patch releases in
**0.4.x–0.10.x** reopen format v1 without migration. Pre-0.4 experimental
directories without metadata require `Model::migrate_legacy_store(path)`.

Standards RDF—not a copied Fjall directory—is the archival continuity contract
across future major-format changes. Read the changelog before minor upgrades and
follow the [production upgrade runbook](rust-production.md#upgrade-runbook).

## Capacity and process ownership

Persistent models retain a complete in-memory query working set in addition to
durable files. Plan both memory and disk capacity. Streaming reads avoid a
second collection but do not turn the model into a disk-only database.

Treat one persistent path as application-owned local mutable state. Do not rely
on multiple independent processes opening the same writable directory as a
coordination protocol. Use one owning service when multiple network clients
must share the store.

## Failure handling

| Error | Response |
|---|---|
| `Error::OpenStore { path, message }` | Fail readiness and inspect path, permissions, mount, and format |
| `Error::Storage(message)` | Stop assuming the write or sync succeeded; preserve context and alert |
| `Error::Unsupported(message)` | Correct read-only, nesting, backend, or operation configuration |
| `Error::Io(error)` | Inspect import/export path and filesystem state |

Never fall back silently from a configured persistent model to memory. Preserve
error chaining and avoid logging sensitive RDF values.

See [Rust production operations](rust-production.md) for deployment,
concurrency, observability, security, and upgrade guidance.
