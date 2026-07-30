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
**not** C source/ABI compatible in 0.2, and not a 100% `librdf` port. crates.io
describes Redland-*shaped* workflows, not drop-in parity.

## What does “Verified” mean in the parity ledger?

It means the curated inventory rows for that subsystem have linked
implementation and tests for the **stated scope**—not that every Redland symbol
or differential fixture passes. Read the inventory JSON and milestone report.

## Parse left data in my model / on disk

You used `Parser::load_into` (progressive). On failure, already-inserted quads
remain (and on Fjall they are durable). Use `load_collecting` for
parse-then-insert batching, or clear the model/store and retry. See [io.md](io.md).

## `text/plain` / `.xml` / `guess` returns Unsupported

Intentional (ADR-008). Pick an explicit `Syntax` or an unambiguous media type /
extension (`.nt`, `.ttl`, `application/rdf+xml`, …).

## Named graphs vanished / parse error on N-Quads

Default `GraphTarget::DefaultGraph` **rejects** named-graph input. Use
`GraphTarget::Dataset` for TriG/N-Quads datasets.

## Update / richer query workflows missing?

ASK and SELECT execution exist today. Oxigraph may also accept CONSTRUCT /
DESCRIBE query text through `Query::execute`, but Oxiland does not yet provide
Redland-shaped result helpers for those forms—treat that as an engine escape
hatch until **0.3**. SPARQL Update is also **0.3**.
Track [milestones/0.3.md](../milestones/0.3.md).

## Where do I report bugs or security issues?

- General bugs and questions: GitHub Issues
- Security: [security policy](../security.md) (private email, not public issues)
- Conduct: [code of conduct](../code-of-conduct.md)

## Performance guidance?

Not published yet as a budgeted suite. Prefer streaming parse/serialize APIs for
large data; avoid collecting full iterators. Fjall mode keeps a full in-memory
working set—plan RAM accordingly.
