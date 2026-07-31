# Python production operations

This guide covers the operational contract around a persistent `Model`.
Oxiland is an embedded RDF database: it runs inside the application process and
stores data in a local directory. It does not provide a network server,
authentication, replication, or a managed backup service.

## Deployment checklist

- Pin an Oxiland version and deploy a wheel matching the exact CPython and
  platform tags.
- Give each persistent dataset a dedicated, trusted directory.
- Open existing production stores with `create=False` so a missing mount fails
  loudly.
- Use transactions for multi-statement writes and transactional loads for
  atomic imports.
- Export N-Quads backups and test restore before upgrades.
- Bound application-facing SPARQL and avoid unrestricted untrusted queries.
- Catch typed exceptions at service boundaries and preserve exception chaining.
- Monitor disk capacity, operation latency, import counts, and failure counts in
  the host application.
- Capacity-test memory as well as disk for the expected dataset and query mix.

## Store lifecycle

Create a persistent model during application startup and keep it for the
process lifetime:

```python
from pathlib import Path

from oxiland import Model, OpenStoreError

STORE_PATH = Path("/srv/app-data/catalog")

try:
    model = Model.open(STORE_PATH, create=False)
except OpenStoreError as error:
    raise SystemExit(f"cannot open RDF store {error.path}: {error.message}") from error
```

Provision a new store as an explicit deployment or initialization step with
`create=True`. Treat failure to open a store as a readiness failure rather than
falling back to a new in-memory model.

A store path is local mutable state. Do not place it on an untrusted or
ephemeral filesystem, and do not assume that sharing one writable directory
between independent processes is a supported coordination mechanism. If an
application needs multiple network writers, place the model behind one service
that owns the store and define concurrency at that service boundary.

## Write atomicity and durability

Group related mutations in a transaction:

```python
with model.transaction() as tx:
    tx.clear_graph(staging_graph)
    for statement in replacement:
        tx.add(statement, graph=staging_graph)
```

Normal context exit commits the complete operation set; exceptional exit
discards it. Nested transactions on one model are unsupported. A persistent
transaction commits through the storage backend. Call `sync()` at explicit
durability boundaries and before controlled shutdown, backup, or handoff.

For imports that must be all-or-nothing, use:

```python
from oxiland import load_path

loaded = load_path(model, "incoming.ttl", transactional=True)
```

Progressive imports can leave successfully parsed statements in the store when
a later statement fails. Use them only when partial progress is an intentional
part of the recovery design.

## Read-only processes

```python
reader = Model.open(STORE_PATH, read_only=True, create=False)
```

Read-only mode prevents mutation through that model and is appropriate for
inspection, validation, and controlled reporting jobs. It is not a substitute
for operating-system permissions: protect the store directory separately.

## Capacity and resource planning

Persistent models keep an in-memory RDF working set for query execution in
addition to the durable files. Size application memory for the loaded dataset,
query intermediates, concurrent work, and normal runtime overhead; disk size
alone is not a sufficient capacity estimate.

Streaming `find`, parse, SELECT, CONSTRUCT, and DESCRIBE results avoids a second
application-level collection, but it does not make the model itself disk-only.
Benchmark representative data and queries on the deployment architecture and
set container or service limits from measured high-water marks. The project
does not publish a universal statements-per-byte or latency guarantee.

## Backup and restore

N-Quads is the portable backup format because it preserves named graphs:

```python
from pathlib import Path

backup = Path("backups/catalog-2026-07-31.nq")
model.sync()
model.export_nquads(backup)
```

Restore into a newly provisioned or intentionally cleared model:

```python
restored = Model.open("var/restored-catalog")
count = restored.import_nquads(backup)
restored.sync()
```

`import_nquads()` merges with existing data; it does not replace the dataset.
Keep backups outside the live store directory, apply normal retention and
access controls, and periodically verify a restore in an isolated path.

## Upgrade runbook

Before changing Oxiland versions in a persistent deployment:

1. Read the changelog and support policy for the target version.
2. Stop or quiesce writers.
3. Call `sync()` and export a full N-Quads backup.
4. Record the current package version and store path.
5. Upgrade in a staging environment and open a copy of the store with
   `create=False`.
6. Run representative ASK/SELECT checks and compare expected statement counts.
7. Upgrade production, keeping the portable backup until validation completes.

Store format v1 is reopen-compatible across 0.4.x–0.8.x patch lines. Very old
experimental stores without format metadata require
`Model.migrate_legacy_store(path)`. Migration and restore should be controlled
maintenance operations, never request-path fallbacks.

## Failure handling

All domain failures inherit from `OxilandError`:

| Exception | Operational meaning |
|---|---|
| `InvalidRdfError` | Invalid IRI, language tag, or RDF term supplied by the caller |
| `ParseError` | Malformed RDF input; inspect `.location` and `.message` |
| `SerializeError` | Dataset cannot be serialized with the requested configuration |
| `SparqlParseError` | Invalid SPARQL text |
| `SparqlEvaluationError` | Query or update execution failed |
| `OpenStoreError` | Store could not be opened; inspect `.path` and `.message` |
| `StorageError` | Persistent read, write, transaction, or sync failed |
| `IoError` | Filesystem input or output failed |
| `UnsupportedError` | Requested syntax, nesting, or operation is outside the API contract |

Validate user input at the edge, but keep Oxiland exceptions as the final
authority. Translate them into application errors without discarding the
original exception:

```python
from oxiland import IoError, ParseError, StorageError, load_path

try:
    loaded = load_path(model, upload_path, transactional=True)
except ParseError as error:
    raise InvalidUpload(f"{error.location}: {error.message}") from error
except (IoError, StorageError) as error:
    raise StoreUnavailable("RDF import failed") from error
```

## Observability

The Python package does not configure application logging. Instrument the
operations that define your service-level behavior:

- store-open success and failure;
- transaction, update, query, import, export, and sync latency;
- statements imported or exported;
- exception counts by concrete Oxiland exception class;
- store-volume free space and backup age.

Avoid logging complete RDF documents, SPARQL text, or literal values by default;
they may contain sensitive application data. Log stable operation names,
durations, counts, and sanitized identifiers instead.

## Security boundaries

Treat store paths, import paths, and backup paths as trusted configuration.
Resolve and authorize user-selected files in the application before passing
them to Oxiland. RDF parsing and SPARQL evaluation are data processing, not
tenant isolation mechanisms; apply request size, query complexity, time, and
process isolation controls appropriate to the deployment.

Report vulnerabilities privately using the [security policy](../security.md).
Operational help and version coverage are described in the
[support policy](../support.md).
