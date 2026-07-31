# FAQ and troubleshooting

## Why does install fail on Rust 1.75 / 1.80?

Oxiland’s MSRV is **1.87** (edition 2024) so CI and Oxigraph 0.5.9 stay aligned.
Upgrade with `rustup update stable`.

## Why does `pip install oxiland` try to build from source?

0.7.0 publishes **wheels only** (no sdist). If pip cannot find a wheel for your
platform/Python, it may attempt a source build and fail. Use CPython 3.10–3.13
on a platform with published wheels, or build from a git checkout with maturin
([Python guide](python.md)).

## Why not use Oxigraph directly?

If you do not need Redland-shaped APIs, inventories, or migration mapping, use
[Oxigraph](https://oxigraph.org/). Oxiland adds a compatibility-oriented facade
and evidence process on top of the same engine. See
[positioning](../evaluators/positioning.md).

## Is Oxiland “Redland-compatible”?

Only in evidence-scoped senses documented in the [parity ledger](../parity.md).
It is **not** C source/ABI compatible (planned 0.8+), and not a 100% `librdf`
port. “Safe-API accounting” means inventoried symbols are **classified**, not
that behavior is drop-in. crates.io describes Redland-*shaped* workflows.

## What does “Verified” mean in the parity ledger?

It means the curated inventory rows for that subsystem have linked
implementation and tests for the **stated scope**—not that every Redland symbol
or differential fixture passes. Read the inventory JSON and milestone report.

## Parse left data in my model / on disk

You used `Parser::load_into` (progressive). On failure, already-inserted quads
remain (and on Fjall they are durable). Use `load_transactional` for atomic
import, `load_collecting` for parse-then-insert batching without a store
transaction, or clear the model/store and retry. See [io.md](io.md).

`Model::import_nquads_from_path` merges into the existing model; it does not
replace the store.

## `text/plain` / `.xml` / `guess` returns Unsupported

Intentional. Pick an explicit `Syntax` or an unambiguous media type /
extension (`.nt`, `.ttl`, `application/rdf+xml`, …).

## Named graphs vanished / parse error on N-Quads

Default `GraphTarget::DefaultGraph` **rejects** named-graph input. Use
`GraphTarget::Dataset` for TriG/N-Quads datasets.

## Fjall store uses a lot of RAM

Fjall mode keeps a full Oxigraph **in-memory working set** for querying. Plan
RAM for the dataset size; use streaming parse/serialize APIs for large files.

## Transaction methods do nothing / buffer forever (Python)

Call `add` / `remove` inside `with model.transaction() as txn:` — methods
outside an entered context raise `UnsupportedError`.

## Will my 0.7 store open in a later 0.7.x?

Format v1 reopen is promised for patch releases in **0.4.x–0.7.x**. See
[persistence](persistence.md).

## Where do I report bugs or security issues?

- General bugs and questions: GitHub Issues
- Security: [security policy](../security.md) (private email, not public issues)
- Support expectations: [support](../support.md)
- Conduct: [code of conduct](../code-of-conduct.md)

## Performance guidance?

Not published yet as a budgeted suite. Prefer streaming parse/serialize APIs for
large data; avoid collecting full iterators. Fjall mode keeps a full in-memory
working set—plan RAM accordingly.
