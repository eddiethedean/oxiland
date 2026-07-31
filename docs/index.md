# Oxiland documentation

Safe Rust and Python APIs for RDF models, named graphs, SPARQL, and streaming
RDF I/O—on pinned Oxigraph.

| Path | Time | Link |
|---|---|---|
| Rust in five minutes | Install + ASK | [Getting started](users/getting-started.md) |
| Python in five minutes | `pip install` + ASK | [Python](users/python.md) |
| Copy-paste examples | Rust + Python | [Examples](users/examples.md) |
| API reference | rustdoc / stubs | [docs.rs](https://docs.rs/oxiland) · [Python API](users/python-api.md) |

Compatibility claims are evidence-scoped. See the [parity ledger](parity.md).

## Users

1. [Getting started](users/getting-started.md) — Rust bootstrap and first workflows
2. [Python](users/python.md) — PyPI package (`pip install oxiland`)
3. [Python API](users/python-api.md) — stubs and key types
4. [Examples](users/examples.md) — runnable Rust and Python scripts
5. [RDF I/O](users/io.md) — syntaxes, GraphTarget, progressive vs collecting load
6. [SPARQL](users/sparql.md) — Query, Update, and results
7. [Persistence](users/persistence.md) — Fjall format v1, transactions, archival export
8. [CLI](users/cli.md) — `oxiland-cli` rdfproc-shaped workflows
9. [Streams](users/streams.md) — fallible iterators and early stop
10. [Utilities and logging](users/utilities.md) — digests, URI helpers, World logs
11. [FAQ and troubleshooting](users/faq.md)

## Evaluators

1. [Positioning](evaluators/positioning.md) — vs Oxigraph, Sophia, and Redland
2. [Migration from Redland](evaluators/migration-from-redland.md)
3. [Redland symbol map](evaluators/redland-symbol-map.md)
4. [Parity ledger](parity.md) — scoped verified claims
5. [Current compatibility report (0.7)](reports/0.7.md)
6. [Compatibility contract](COMPATIBILITY.md)

## Contributors

1. [Contributing](contributing.md) — fast path and compatibility slices
2. [Project charter](CHARTER.md)
3. [Roadmap](ROADMAP.md) — next release is **0.8** (C ABI preview)
4. [Execution plan](EXECUTION.md)
5. [Architecture](ARCHITECTURE.md)
6. [Verification](VERIFICATION.md)
7. [Decisions](DECISIONS.md)
8. [Risks](RISKS.md)

### Document authority

| Question | Authority |
|---|---|
| Who is Oxiland for and what does 1.0 promise? | [Charter](CHARTER.md) |
| What exists and is verified now? | [Parity ledger](parity.md) |
| What release comes next? | [Roadmap](ROADMAP.md) (**0.8**) |
| How is work sliced and completed? | [Execution](EXECUTION.md) |
| Where does code belong? | [Architecture](ARCHITECTURE.md) |
| What does compatibility mean? | [Compatibility](COMPATIBILITY.md) |
| What evidence is sufficient? | [Verification](VERIFICATION.md) |
| Why was a durable choice made? | [Decisions](DECISIONS.md) |
| What could block the plan? | [Risks](RISKS.md) |
| Support / deprecation for 0.x? | [Support](support.md) |

## Project

- [Support](support.md)
- [Security](security.md)
- [Code of conduct](code-of-conduct.md)

Historical reports, release checklists, completed milestones, and design notes
live under **Archive** in the site navigation.

## Maintenance rules

- Plans use `planned`, `in progress`, `blocked`, and `complete`.
- Only verified implementation is marked complete in the parity ledger.
- User and evaluator guides must not describe planned work as available.
- Historical reports are amended only to correct errors.
