# FAQ and troubleshooting

## Why does install fail on Rust 1.75 / 1.80?

Oxiland’s MSRV is **1.87** (edition 2024) so CI and Oxigraph 0.5.9 stay aligned.
Upgrade with `rustup update stable`.

## Why not use Oxigraph directly?

If you do not need Redland-shaped APIs, inventories, or migration mapping, use
[Oxigraph](https://oxigraph.org/). Oxiland adds a compatibility-oriented facade
and evidence process on top of the same engine. See
[positioning](../evaluators/positioning.md).

## Is Oxiland “Redland-compatible”?

Only in evidence-scoped senses documented in the [parity ledger](../parity.md). It is
**not** C source/ABI compatible in 0.5, and not a 100% `librdf` port. crates.io
describes Redland-*shaped* workflows, not drop-in parity.

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

Intentional (ADR-008). Pick an explicit `Syntax` or an unambiguous media type /
extension (`.nt`, `.ttl`, `application/rdf+xml`, …).

## Named graphs vanished / parse error on N-Quads

Default `GraphTarget::DefaultGraph` **rejects** named-graph input. Use
`GraphTarget::Dataset` for TriG/N-Quads datasets.

## Update / richer query workflows missing?

ASK, SELECT, CONSTRUCT, DESCRIBE, Update, dataset selection, limit/offset, and
SPARQL Results serialization shipped in **0.3**. Durable on-disk contracts and
storage transactions shipped in **0.4**. Utilities, digests, vocabulary helpers,
and World logging shipped in **0.5**. A Pythonic PyPI package shipped in
**0.7** (`pip install oxiland`; not a 1:1 Rust port — see
[Python guide](python.md)). C ABI preview is planned for **0.8**. Track
[milestones](../milestones/0.7.md) and the [roadmap](../ROADMAP.md).

## Where do I report bugs or security issues?

- General bugs and questions: GitHub Issues
- Security: [security policy](../security.md) (private email, not public issues)
- Conduct: [code of conduct](../code-of-conduct.md)

## Performance guidance?

Not published yet as a budgeted suite. Prefer streaming parse/serialize APIs for
large data; avoid collecting full iterators. Fjall mode keeps a full in-memory
working set—plan RAM accordingly.
