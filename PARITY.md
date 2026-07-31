# Redland parity ledger

Last completed milestone: 0.7
Current development milestone: 0.8 (`planned`)
Ledger maturity: header-derived full public `librdf` 1.0.17 function inventory
(0.6); Python package usability evidence (0.7); curated 0.1–0.5 slices retained
for historical evidence

Target: the documented Redland `librdf` 1.0.17 API (manual labeled 1.0.18).

Planned sequencing and completion rules are documented in the
[0.x roadmap](docs/ROADMAP.md) and
[compatibility plan](docs/COMPATIBILITY.md).

Inventory revisions:

- [`compatibility/inventory/redland-1.0.17-oxiland-0.1.json`](compatibility/inventory/redland-1.0.17-oxiland-0.1.json)
- [`compatibility/inventory/redland-1.0.17-oxiland-0.2.json`](compatibility/inventory/redland-1.0.17-oxiland-0.2.json)
- [`compatibility/inventory/redland-1.0.17-oxiland-0.3.json`](compatibility/inventory/redland-1.0.17-oxiland-0.3.json)
- [`compatibility/inventory/redland-1.0.17-oxiland-0.4.json`](compatibility/inventory/redland-1.0.17-oxiland-0.4.json)
- [`compatibility/inventory/redland-1.0.17-oxiland-0.5.json`](compatibility/inventory/redland-1.0.17-oxiland-0.5.json)
- [`compatibility/inventory/redland-1.0.17-oxiland-0.6.json`](compatibility/inventory/redland-1.0.17-oxiland-0.6.json)

0.6 compatibility report: [`docs/reports/0.6.md`](docs/reports/0.6.md)

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
| World / lifecycle | Verified (0.6 accounting) | Unstarted | 0.1/0.8 | RAII world, features, log handlers (ADR-014) |
| URI | Verified (0.6 accounting) | Unstarted | 0.5/0.8 | join/relativize/file-URI helpers |
| Nodes | Verified (0.6 accounting) | Unstarted | 0.1/0.8 | Oxigraph term re-exports plus InvalidRdf helpers (ADR-004) |
| Statements | Verified (0.6 accounting) | Unstarted | 0.1/0.8 | triples and `StatementPattern` matching |
| Model | Verified (0.6 accounting) | Unstarted | 0.1/0.8 | default and named-graph CRUD, size, streaming find |
| Storage | Verified (0.6 accounting) | Unstarted | 0.4/0.9 | format v1 Fjall; plugins excluded (ADR-018) |
| Streams / iterators | Verified (0.6 accounting) | Unstarted | 0.5/0.8 | find/parse/query streams; ADR-013 policy |
| Parser | Verified (0.6 accounting) | Unstarted | 0.2/0.8 | `Parser` facade, Syntax discovery |
| Serializer | Verified (0.6 accounting) | Unstarted | 0.2/0.8 | `Serializer` facade |
| SPARQL query/results | Verified (0.6 accounting) | Unstarted | 0.3/0.8 | Query / results / ResultsFormat |
| Query update | Verified (0.6 accounting) | Unstarted | 0.3/0.8 | `Update` facade |
| Digests | Verified (0.6 accounting) | Unstarted | 0.5/0.8 | MD5/SHA-1/SHA-256 (ADR-015) |
| Hashes / lists | Dispositioned | Unstarted | 0.5/0.8 | `not-applicable` → `HashMap`/`Vec` (ADR-016) |
| Heuristics / files / Unicode | Verified (0.6 accounting) | Unstarted | 0.5/0.8 | file URI + NFC/NFKC helpers |
| Logging | Verified (0.6 accounting) | Unstarted | 0.5/0.8 | World handlers + optional `tracing` |
| Storage plug-ins | Dispositioned | Unstarted | 0.4/0.9 | excluded / Unsupported |
| `rdfproc` utility | Verified (0.6 CLI) | n/a | 0.6 | `oxiland-cli` workflows (ADR-019) |

## Safe-API accounting (0.6)

**100% safe-API accounting** for header-derived public `librdf_*` functions:
383 classified (238 verified, 96 not-applicable, 49 excluded); 0 unreviewed.
See [`docs/reports/0.6.md`](docs/reports/0.6.md).

## Current evidence

- Inventory: full 0.6 header-derived manifest + curated 0.1–0.5 slices.
- Integration tests + `tests/accounting.rs` + `oxiland-cli` tests.
- CLI smoke: `compatibility/harness/cli_smoke.py`.
- Public API snapshot + `cargo semver-checks` vs 0.6.0.
- ADR-004–ADR-021 accepted as applicable.
- Oxigraph 0.5.9 remains pinned with default features disabled.

## Next ledger upgrade

Begin 0.8 C ABI preview accounting; keep Python package evidence current on
main.
