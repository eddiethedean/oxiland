# Oxiland documentation

Production-oriented Python and Rust APIs for RDF models, named graphs, SPARQL,
persistence, and streaming RDF I/O.

| Path | Time | Link |
|---|---|---|
| Python in five minutes | `pip install` + ASK | [Python](users/python.md) |
| Deploy the Python package | Storage, backups, failures, upgrades | [Production operations](users/python-production.md) |
| Python API | Typed public surface | [API reference](users/python-api.md) |
| Rust in five minutes | Install + ASK | [Rust overview](users/rust.md) |
| Deploy a Rust service | Storage, concurrency, backups, query budgets | [Production operations](users/rust-production.md) |
| Command line | Imports, inspection, queries, scripting | [CLI guide](users/cli.md) |
| Copy-paste examples | Rust + Python | [Examples](users/examples.md) |
| Rust API | rustdoc | [docs.rs](https://docs.rs/oxiland) |

Compatibility claims are evidence-scoped. See the [parity ledger](parity.md).

## Python

1. [Overview](users/python.md) — install, first model, and package capabilities
2. [Installation and compatibility](users/python-installation.md) — wheels, reproducible deployment, and source builds
3. [Models and RDF terms](users/python-models.md) — values, graphs, CRUD, matching, and transactions
4. [RDF I/O and SPARQL](users/python-data.md) — syntax support, import modes, queries, and updates
5. [Production operations](users/python-production.md) — lifecycle, durability, backups, upgrades, and security boundaries
6. [API reference](users/python-api.md) — public classes, functions, signatures, and errors

## Rust

1. [Overview](users/rust.md) — public surface, models, storage, and errors
2. [Getting started](users/getting-started.md) — project bootstrap and first workflows
3. [RDF I/O](users/io.md) — syntaxes, graph targets, and import semantics
4. [SPARQL](users/sparql.md) — queries, updates, datasets, and result formats
5. [Persistence](users/persistence.md) — format v1, transactions, and archival export
6. [Streams](users/streams.md) — fallible iterators, ownership, and early stop
7. [Utilities and logging](users/utilities.md) — digests, IRIs, namespaces, and observability
8. [Production operations](users/rust-production.md) — lifecycle, concurrency, capacity, backup, and upgrades
9. [Rust API reference](https://docs.rs/oxiland)

## Command line

1. [CLI guide](users/cli.md) — install, store selection, commands, automation, and failure behavior
2. [Examples](users/examples.md) — runnable Python and Rust programs
3. [FAQ and troubleshooting](users/faq.md)

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
