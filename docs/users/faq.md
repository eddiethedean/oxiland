# FAQ and troubleshooting

## Why does install fail on Rust 1.75 / 1.80?

Oxiland’s MSRV is **1.87** (edition 2024) so CI and Oxigraph 0.5.9 stay aligned.
Upgrade with `rustup update stable`.

## Why does `pip install oxiland` try to build from source?

Published packages are **wheels only** (no sdist). If pip cannot find a wheel
for your platform/Python, it may attempt a source build and fail. Use CPython
3.10–3.14 on a platform with published wheels, or build from a git checkout with
maturin ([Python installation guide](python-installation.md)).

Tip **0.12.0** is the current package version.

## Why not use Oxigraph directly?

If you do not need Redland-shaped APIs, inventories, or migration mapping, use
[Oxigraph](https://oxigraph.org/). Oxiland adds a compatibility-oriented facade
and evidence process on top of the same engine. See
[positioning](../evaluators/positioning.md).

## Is Oxiland a network database?

No. In Rust and Python, a persistent model is an embedded local store inside
the application process. Oxiland does not provide a server, authentication,
replication, tenant isolation, or managed backups. See the
[Rust](rust-production.md) or [Python](python-production.md) production guide.

## Is Oxiland “Redland-compatible”?

Only in evidence-scoped senses documented in the [parity ledger](../parity.md).
Tip **0.12** retains the **0.11** demonstrated Redland matrix for Rust, Python,
and C source + librdf-compat packaging. Remaining behavioral gaps are listed in
[C ABI limitations](c-abi-limitations.md). Oxiland is not a silent 100%
`librdf` port and not an rdflib adapter. “Safe-API accounting” means inventoried
symbols are **classified**, not that every Redland behavior is drop-in. See the
[C ABI guide](c-abi.md).

## Does Oxiland ship a C package on crates.io?

No. `oxiland-capi` is `publish = false`. Build it from a repository checkout
(`cargo build -p oxiland-capi`). See [C ABI](c-abi.md).

## What does “Verified” mean in the parity ledger?

It means the curated inventory rows for that subsystem have linked
implementation and tests for the **stated scope**—not that every Redland symbol
or differential fixture passes. Read the inventory JSON and milestone report.

## Parse left data in my model / on disk

You used a progressive load. On failure, already-inserted quads remain (and on
Fjall they are durable). In Python, use
`load_path(model, path, transactional=True)` for atomic import or keep the
default `collecting=True` for parse-then-insert behavior. In Rust, use
`load_transactional` or `load_collecting`. See the
[Python data guide](python-data.md) or [Rust I/O guide](io.md).

`Model::import_nquads_from_path` merges into the existing model; it does not
replace the store.

## `text/plain` / `.xml` / `guess` returns Unsupported

Intentional. Pick an explicit `Syntax` or an unambiguous media type /
extension (`.nt`, `.ttl`, `application/rdf+xml`, …).

## Named graphs vanished / parse error on N-Quads

The default graph target **rejects** named-graph input. Rust callers can use
`GraphTarget::Dataset` for TriG/N-Quads datasets. The Python API does not
expose that target; load one compatible graph with `graph=` or create named
graphs programmatically. See [Python RDF I/O](python-data.md#stream-a-document).

The CLI has the same dataset-import limitation. Restore multi-graph N-Quads
backups with Rust `Model::import_nquads_from_path` or Python
`Model.import_nquads`. Rust callers can also use
`Parser::parse_path_with_extension` or configure `GraphTarget::Dataset`.

## Wrong, unknown, or not-compiled storage backends

Backend selection is explicit and fails closed:

- **Unknown** names raise `Unsupported` / `UnsupportedError` (“not recognized”).
- **Known but not compiled** optional names (for example future adapters such as
  redb) raise a distinct “known but not compiled into this build” error—they
  never fall back to Fjall or memory.
- **Compiled** backends for the default build are `memory` and `fjall`.

Use `compiled_backends()` (Rust and Python) to list what this build links.
`storage_backend_available(name)` returns `True` only for compiled backends;
other names error as above. CLI `--storage` and C `librdf_new_storage` follow
the same registry.

## Fjall store uses a lot of RAM

Fjall mode keeps a full Oxigraph **in-memory working set** for querying. Plan
RAM for the dataset size; use streaming parse/serialize APIs for large files.
See [Performance](performance.md).

## Transaction methods do nothing / buffer forever (Python)

Call `add` / `remove` inside `with model.transaction() as txn:` — methods
outside an entered context raise `UnsupportedError`.

## Will my format-v1 store open after an upgrade?

Format v1 reopen is promised for patch releases in **0.4.x–0.12.x**. Export
N-Quads before any future format-v2 migration. See
[persistence](persistence.md) and the [support policy](../support.md).

## Can multiple processes write the same store path?

Do not use a shared writable directory as an inter-process coordination
mechanism. Keep one owning application/service per store and expose controlled
network concurrency above it. `Model` sharing and locks are in-process
guarantees.

## Where do I report bugs or security issues?

- General bugs and questions: [GitHub Issues](https://github.com/eddiethedean/oxiland/issues)
- Security: [security policy](../security.md) (private email, not public issues)
- Support expectations: [support](../support.md)
- Conduct: [code of conduct](../code-of-conduct.md)

## Performance guidance?

See [Performance](performance.md) for published suites, host scope, and what is
not claimable. Prefer streaming parse/serialize APIs for large data; avoid
collecting full iterators. Fjall mode keeps a full in-memory working set—plan
RAM accordingly.
