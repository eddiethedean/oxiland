# Redland parity ledger

Last completed milestone: 0.2
Current development milestone: 0.3 (`planned`)
Ledger maturity: curated 0.1 core and 0.2 I/O inventory slices; full
header-derived generation pending with the broader oracle harness

Target: the documented Redland `librdf` 1.0.17 API (manual labeled 1.0.18).

Planned sequencing and completion rules are documented in the
[0.x roadmap](docs/ROADMAP.md) and
[compatibility plan](docs/COMPATIBILITY.md).

Inventory revisions:

- [`compatibility/inventory/redland-1.0.17-oxiland-0.1.json`](compatibility/inventory/redland-1.0.17-oxiland-0.1.json)
- [`compatibility/inventory/redland-1.0.17-oxiland-0.2.json`](compatibility/inventory/redland-1.0.17-oxiland-0.2.json)

0.2 compatibility report: [`docs/reports/0.2.md`](docs/reports/0.2.md)

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
| World / lifecycle | Verified (0.1 slice) | Unstarted | 0.1/0.7 | RAII world and feature registry |
| URI | Implemented | Unstarted | 0.1/0.7 | validated named nodes; helper parity beyond construction open |
| Nodes | Verified (0.1 slice) | Unstarted | 0.1/0.7 | Oxigraph term re-exports plus InvalidRdf helpers (ADR-004) |
| Statements | Verified (0.1 slice) | Unstarted | 0.1/0.7 | triples and `StatementPattern` matching |
| Model | Verified (0.1 slice) | Unstarted | 0.1/0.7 | default and named-graph CRUD, size, streaming find |
| Storage | Partial | Unstarted | 0.4/0.8 | memory default; fjall persistence via `Model::open` |
| Streams / iterators | Verified (find + parse) | Unstarted | 0.5/0.7 | `StatementMatches` and parser streams |
| Parser | Verified (0.2 slice) | Unstarted | 0.2/0.7 | `Parser` facade, Syntax discovery, progressive/collecting load |
| Serializer | Verified (0.2 slice) | Unstarted | 0.2/0.7 | `Serializer` facade, prefixes, graph/dataset checks |
| SPARQL query/results | Partial | Unstarted | 0.3/0.7 | ASK/SELECT execution; parse vs evaluation errors |
| Query update | Unstarted | Unstarted | 0.3/0.7 | Oxigraph capability not yet exposed |
| Digests | Unstarted | Unstarted | 0.5/0.7 | inventory and mapping pending |
| Hashes / lists | Unreviewed | Unstarted | 0.5/0.7 | likely Rust replacements; rationale required |
| Heuristics / files / Unicode | Partial (I/O Unicode) | Unstarted | 0.5/0.7 | Unicode literals covered in 0.2 I/O tests |
| Logging | Unstarted | Unstarted | 0.5/0.7 | callback and `tracing` design pending |
| Storage plug-ins | Unreviewed | Unstarted | 0.4/0.8 | per-backend decisions required |
| `rdfproc` utility | Unstarted | n/a | 0.6 | CLI workflow inventory pending |

## Current evidence

- Inventory: 22 curated 0.1 entries (18 verified, 4 implemented) plus 10
  curated 0.2 I/O entries (10 verified).
- Integration tests cover world features, CRUD, named graphs, streaming find,
  SPARQL ASK/SELECT, invalid input, unsupported storage, and the full 0.2 I/O
  acceptance matrix in `tests/io.rs`.
- Curated W3C-style syntax cases run through the public facade
  (`tests/conformance.rs`).
- Native `rapper` oracle and differential smoke harnesses are available under
  `compatibility/harness/`.
- Examples `quick_start`, `contexts`, `parse_serialize`, `select`, and
  `progressive_load` run in CI.
- ADR-004, ADR-005, ADR-007, and ADR-008 are accepted.
- Oxigraph 0.5.9 remains pinned with default features disabled.

## Next ledger upgrade

Generate the remaining Redland symbols from pinned headers once the broader
oracle harness expands beyond the I/O subset. Expand verified rows only when
differential or standards fixtures exist for the claimed behavior.

“100% parity” is reached only when every public Redland function is represented
in a generated symbol inventory, has a documented mapping or intentional
safe-Rust replacement, and satisfies the evidence required for the specific
compatibility claim. No blended percentage is used.
