# Rust production operations

Oxiland runs inside the application process. A persistent `Model` combines an
in-memory query working set with a durable Fjall-backed format-v1 store. It is
not a network database and does not provide authentication, replication,
tenant isolation, or managed backups.

## Deployment checklist

- Pin the Oxiland crate and Rust toolchain in the application's lockfile.
- Give each dataset a dedicated, trusted local directory.
- Use `OpenOptions::create(false)` when production must fail on a missing mount.
- Keep one long-lived model handle and clone it only to share the same dataset.
- Use transactions for related writes and transactional loads for atomic imports.
- Export N-Quads backups and verify restore before upgrades.
- Bound SPARQL work and attach cancellation tokens to caller-facing queries.
- Capacity-test memory, disk, query intermediates, and concurrent workload.
- Preserve typed errors and instrument operation latency and failure categories.

## Startup and readiness

```rust,no_run
use oxiland::{Model, OpenOptions};

fn open_production_store() -> oxiland::Result<Model> {
    Model::open_with(
        OpenOptions::fjall("/srv/app-data/catalog")
            .create(false)
            .read_only(false),
    )
}
```

Provision a new store explicitly with `create(true)`. Do not fall back to an
empty in-memory model when a configured persistent store fails to open; treat
that as a readiness failure.

`Model::clone()` shares the same underlying dataset and lock state. It is not a
snapshot or deep copy. Use clones for application ownership where shared access
is intended.

## Atomic writes

```rust,no_run
use oxiland::terms::{Literal, Triple, named_node};
use oxiland::Model;

fn write(model: &Model) -> oxiland::Result<()> {
    model.transaction(|tx| {
        tx.add(Triple::new(
            named_node("https://example.com/item/42")?,
            named_node("https://example.com/status")?,
            Literal::new_simple_literal("active"),
        ))?;
        Ok(())
    })
}
```

The callback commits as one unit when it returns `Ok`. Returning `Err` rolls
back. Nested transactions and auto-commit mutation during an active transaction
return `Error::Unsupported`. Reads from the transaction-owning thread see the
last committed working set, not uncommitted changes.

Persistent transaction commits synchronize the durable store. Call `sync()` at
explicit shutdown, backup, and handoff boundaries and handle any failure.

## Concurrency and ownership

`Model` is `Send + Sync`; reads take a shared lock and writes take an exclusive
lock. This is an in-process guarantee. Do not use a shared writable directory as
an inter-process coordination primitive. If multiple network clients need to
write, put one owning service in front of the model and define concurrency at
that service boundary.

Iterators hold the state needed to continue their operation. Avoid holding a
long-lived read/result iterator while waiting on unrelated work, because that
can extend resource lifetimes and delay application-level progress.

## Query budgets and cancellation

```rust,no_run
use std::thread;
use std::time::Duration;

use oxiland::sparql::CancellationToken;
use oxiland::{Model, Query};

fn bounded_query(model: &Model) -> oxiland::Result<()> {
    let token = CancellationToken::new();
    let cancel = token.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        cancel.cancel();
    });

    let _results = Query::new("SELECT * WHERE { ?s ?p ?o }")
        .limit(1_000)?
        .cancellation_token(token)
        .execute(model)?;
    Ok(())
}
```

Cancellation is cooperative, not a hard real-time guarantee. Also constrain
request size, query form, result count, and worker isolation. Do not execute
unrestricted untrusted SPARQL in a latency-sensitive process.

## Capacity planning

Persistent models keep a full in-memory RDF working set for querying in
addition to durable files. Streaming prevents a second full application-level
collection, but it does not make the model disk-only. Benchmark representative
datasets, queries, imports, and update concurrency on the deployment target.

Monitor at least:

- resident memory and store-volume free space;
- store-open, query, update, transaction, load, export, and sync latency;
- statements processed and result counts;
- errors grouped by `Error` variant;
- backup age and restore-test status.

## Backup and restore

```rust,no_run
use oxiland::Model;

fn backup(model: &Model) -> oxiland::Result<()> {
    model.sync()?;
    model.export_nquads_to_path("./backups/catalog.nq")
}

fn restore() -> oxiland::Result<Model> {
    let model = Model::open("./data/restored-catalog")?;
    model.import_nquads_from_path("./backups/catalog.nq")?;
    model.sync()?;
    Ok(model)
}
```

N-Quads preserves named graphs. Import merges with existing data; it does not
replace the dataset. Restore into a new store or clear an intentionally selected
target first. Keep backups outside the live store directory and test restores
in an isolated path.

## Upgrade runbook

1. Read the changelog and target-version support policy.
2. Quiesce writers and finish in-flight iterators.
3. Call `sync()` and export a complete N-Quads backup.
4. Record the package version, store path, and expected statement counts.
5. Open a copied store with `create(false)` under the new version in staging.
6. Run representative ASK/SELECT checks and a backup/restore smoke test.
7. Upgrade production and retain the portable backup until validation completes.

Format v1 reopens across 0.4.x–0.7.x patch lines. Pre-0.4 experimental stores
without metadata require `Model::migrate_legacy_store`; run migration only as a
controlled maintenance operation.

## Security boundaries

Treat store paths, import paths, and backup paths as trusted configuration.
Authorize user-selected paths before passing them to Oxiland. Limit input size
and isolate expensive parsing or SPARQL evaluation according to the deployment
threat model. Avoid logging raw RDF documents, query text, or literals by
default because they may contain sensitive data.

See the [security policy](../security.md) and [support policy](../support.md).
