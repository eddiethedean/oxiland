# Oxiland documentation

Choose a path by role. Planning documents describe intended work; the
[parity ledger](parity.md) records what is verified now.

API reference (rustdoc): [docs.rs/oxiland](https://docs.rs/oxiland).

## Users

Product guides for writing applications against the published crate:

1. [Getting started](users/getting-started.md) — install, toolchain, first workflows
2. [RDF I/O](users/io.md) — syntaxes, GraphTarget, progressive vs collecting load
3. [SPARQL](users/sparql.md) — Query, Update, and results
4. [Streams](users/streams.md) — fallible iterators and early stop
5. [Utilities and logging](users/utilities.md) — digests, URI helpers, World logs
6. [CLI](users/cli.md) — `oxiland-cli` rdfproc-shaped workflows
7. [Persistence](users/persistence.md) — Fjall format v1, transactions, archival export
8. [FAQ and troubleshooting](users/faq.md)

## Evaluators

Adoption and compatibility decision material:

1. [Positioning](evaluators/positioning.md) — vs Oxigraph, Sophia, and Redland
2. [Migration from Redland](evaluators/migration-from-redland.md)
3. [Redland symbol map](evaluators/redland-symbol-map.md) — 0.6 accounting
4. [Parity ledger](parity.md) — scoped verified claims
5. [0.6 compatibility report](reports/0.6.md)
6. [0.5 compatibility report](reports/0.5.md)
7. [0.4 compatibility report](reports/0.4.md)
8. [0.3 compatibility report](reports/0.3.md)
9. [0.2 compatibility report](reports/0.2.md)
10. [Compatibility contract](COMPATIBILITY.md)

## Contributors

Process and planning (compatibility work is vertical-slice driven):

1. [Contributing](contributing.md)
2. [Project charter](CHARTER.md)
3. [Roadmap](ROADMAP.md) — next release is **0.7**
4. [Milestone 0.6](milestones/0.6.md) (complete) · [0.5](milestones/0.5.md) · [0.4](milestones/0.4.md)
5. [Execution plan](EXECUTION.md)
6. [Architecture](ARCHITECTURE.md)
7. [Verification](VERIFICATION.md)
8. [Decisions](DECISIONS.md)
9. [Risks](RISKS.md)
10. [Reports and release checklists](reports/0.6.md) · [0.6.0 release](reports/0.6.0-release.md) · [0.5](reports/0.5.md)

### Document authority (contributors)

| Question | Authority |
|---|---|
| Who is Oxiland for and what does 1.0 promise? | [Charter](CHARTER.md) |
| What exists and is verified now? | [Parity ledger](parity.md) |
| What release comes next? | [Roadmap](ROADMAP.md) |
| What must the active release deliver? | [milestones/0.6.md](milestones/0.6.md) (complete); next is 0.7 |
| How is work sliced and completed? | [Execution](EXECUTION.md) |
| Where does code belong? | [Architecture](ARCHITECTURE.md) |
| What does compatibility mean? | [Compatibility](COMPATIBILITY.md) |
| What evidence is sufficient? | [Verification](VERIFICATION.md) |
| Why was a durable choice made? | [Decisions](DECISIONS.md) |
| What could block the plan? | [Risks](RISKS.md) |

## Maintenance rules

- Plans use `planned`, `in progress`, `blocked`, and `complete`.
- Only verified implementation is marked complete in the parity ledger.
- User and evaluator guides must not describe planned work as available.
- Historical reports are amended only to correct errors.
- This MkDocs site is configured by [`mkdocs.yml`](https://github.com/eddiethedean/oxiland/blob/main/mkdocs.yml)
  and [`.readthedocs.yaml`](https://github.com/eddiethedean/oxiland/blob/main/.readthedocs.yaml).
