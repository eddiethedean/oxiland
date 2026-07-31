# Redland parity ledger

Last completed milestone: 0.4
Current development milestone: 0.5 (`planned`)
Ledger maturity: curated 0.1 core, 0.2 I/O, 0.3 query/update, and 0.4 storage
inventory slices; full header-derived generation pending with the broader oracle harness

Target: the documented Redland `librdf` 1.0.17 API (manual labeled 1.0.18).

Planned sequencing and completion rules are documented in the
[0.x roadmap](docs/ROADMAP.md) and
[compatibility plan](docs/COMPATIBILITY.md).

Inventory revisions:

- [`compatibility/inventory/redland-1.0.17-oxiland-0.1.json`](compatibility/inventory/redland-1.0.17-oxiland-0.1.json)
- [`compatibility/inventory/redland-1.0.17-oxiland-0.2.json`](compatibility/inventory/redland-1.0.17-oxiland-0.2.json)
- [`compatibility/inventory/redland-1.0.17-oxiland-0.3.json`](compatibility/inventory/redland-1.0.17-oxiland-0.3.json)
- [`compatibility/inventory/redland-1.0.17-oxiland-0.4.json`](compatibility/inventory/redland-1.0.17-oxiland-0.4.json)

0.4 compatibility report: [`docs/reports/0.4.md`](docs/reports/0.4.md)

## Status vocabulary

- `unreviewed`: not yet mapped from canonical Redland inputs.
- `mapped`: intended Rust/C representation is documented.
- `implemented`: code exists but lacks complete compatibility evidence.
- `verified`: required evidence passes for the **stated inventory scope**
  (linked implementation + tests, and fixtures named by that milestone). It does
  **not** mean every Redland symbol or a full native differential suite has
  passed—read the linked inventory revision and milestone report.
- `not-applicable`: legacy mechanism replaced safely with rationale.
- `excluded`: intentionally unsupported with an approved impact assessment.

## Subsystem summary

| Redland subsystem | Safe Rust | C ABI | Target | Current evidence / gap |
|---|---|---|---:|---|
| World / lifecycle | Verified (0.1 slice) | Unstarted | 0.1/0.8 | RAII world and feature registry |
| URI | Implemented | Unstarted | 0.1/0.8 | validated named nodes; helper parity beyond construction open |
| Nodes | Verified (0.1 slice) | Unstarted | 0.1/0.8 | Oxigraph term re-exports plus InvalidRdf helpers (ADR-004) |
| Statements | Verified (0.1 slice) | Unstarted | 0.1/0.8 | triples and `StatementPattern` matching |
| Model | Verified (0.1 slice) | Unstarted | 0.1/0.8 | default and named-graph CRUD, size, streaming find |
| Storage | Verified (0.4 slice) | Unstarted | 0.4/0.9 | format v1 Fjall; transactions, sync, clear, capabilities (ADR-006) |
| Streams / iterators | Verified (find + parse + query) | Unstarted | 0.5/0.8 | `StatementMatches`, parser, and query result streams |
| Parser | Verified (0.2 slice) | Unstarted | 0.2/0.8 | `Parser` facade, Syntax discovery, progressive/collecting load |
| Serializer | Verified (0.2 slice) | Unstarted | 0.2/0.8 | `Serializer` facade, prefixes, graph/dataset checks |
| SPARQL query/results | Verified (0.3 slice) | Unstarted | 0.3/0.8 | Query builder, streaming results, ResultsFormat |
| Query update | Verified (0.3 slice) | Unstarted | 0.3/0.8 | `Update` facade; write-locked Fjall resync with compensated rollback |
| Digests | Unstarted | Unstarted | 0.5/0.8 | inventory and mapping pending |
| Hashes / lists | Unreviewed | Unstarted | 0.5/0.8 | likely Rust replacements; rationale required |
| Heuristics / files / Unicode | Partial (I/O Unicode) | Unstarted | 0.5/0.8 | Unicode literals covered in 0.2 I/O tests |
| Logging | Unstarted | Unstarted | 0.5/0.8 | callback and `tracing` design pending |
| Storage plug-ins | Dispositioned | Unstarted | 0.4/0.9 | legacy names → Unsupported; see `docs/design/0.4-legacy-storage.md` |
| `rdfproc` utility | Unstarted | n/a | 0.6 | CLI workflow inventory pending |

## Current evidence

- Inventory: 22 curated 0.1 entries, 10 curated 0.2 I/O entries, 10 curated
  0.3 query/update entries, and 10 curated 0.4 storage entries (verified in
  their slices).
- Integration tests cover world features, CRUD, named graphs, streaming find,
  SPARQL query/update/results (`tests/query.rs`), storage/transactions
  (`tests/storage.rs`), invalid input, unsupported storage, and the 0.2 I/O
  matrix in `tests/io.rs`.
- Curated W3C-style syntax cases run through the public facade
  (`tests/conformance.rs`).
- Native `rapper` I/O oracle/differential and SPARQL facade smoke harnesses are
  available under `compatibility/harness/`.
- Examples `quick_start`, `contexts`, `parse_serialize`, `select`,
  `progressive_load`, `construct`, `update`, and `persistent_transaction` run
  in CI.
- ADR-004, ADR-005, ADR-006, ADR-007, ADR-008, and ADR-009–ADR-012 are accepted.
- Oxigraph 0.5.9 remains pinned with default features disabled.

## Next ledger upgrade

Generate the remaining Redland symbols from pinned headers once the broader
oracle harness expands. Expand verified rows only when differential or standards
fixtures exist for the claimed behavior. Next development focus: **0.5**
streams/utilities/observability, then safe-API accounting (0.6), the
**Pythonic Python package (0.7)**, and C ABI (0.8+).

“100% parity” is reached only when every public Redland function is represented
in a generated symbol inventory, has a documented mapping or intentional
safe-Rust replacement, and satisfies the evidence required for the specific
compatibility claim. No blended percentage is used.
