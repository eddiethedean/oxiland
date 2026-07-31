# Redland parity ledger

Last completed milestone: 0.5
Current development milestone: 0.6 (`planned`)
Ledger maturity: curated 0.1–0.5 inventory slices; full header-derived
generation pending with the broader oracle harness

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

0.5 compatibility report: [`docs/reports/0.5.md`](docs/reports/0.5.md)

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
| World / lifecycle | Verified (0.1 + 0.5 logging) | Unstarted | 0.1/0.8 | RAII world, features, log handlers (ADR-014) |
| URI | Verified (0.5 helpers) | Unstarted | 0.5/0.8 | join/relativize/file-URI helpers |
| Nodes | Verified (0.1 slice) | Unstarted | 0.1/0.8 | Oxigraph term re-exports plus InvalidRdf helpers (ADR-004) |
| Statements | Verified (0.1 slice) | Unstarted | 0.1/0.8 | triples and `StatementPattern` matching |
| Model | Verified (0.1 slice) | Unstarted | 0.1/0.8 | default and named-graph CRUD, size, streaming find |
| Storage | Verified (0.4 slice) | Unstarted | 0.4/0.9 | format v1 Fjall; transactions, sync, clear, capabilities (ADR-006) |
| Streams / iterators | Verified (0.5 policy) | Unstarted | 0.5/0.8 | find/parse/query streams; ADR-013 policy |
| Parser | Verified (0.2 slice) | Unstarted | 0.2/0.8 | `Parser` facade, Syntax discovery, progressive/collecting load |
| Serializer | Verified (0.2 slice) | Unstarted | 0.2/0.8 | `Serializer` facade, prefixes, graph/dataset checks |
| SPARQL query/results | Verified (0.3 slice) | Unstarted | 0.3/0.8 | Query builder, streaming results, ResultsFormat |
| Query update | Verified (0.3 slice) | Unstarted | 0.3/0.8 | `Update` facade; write-locked Fjall resync with compensated rollback |
| Digests | Verified (0.5 slice) | Unstarted | 0.5/0.8 | MD5/SHA-1/SHA-256 (ADR-015) |
| Hashes / lists | Dispositioned | Unstarted | 0.5/0.8 | `not-applicable` → `HashMap`/`Vec` (ADR-016) |
| Heuristics / files / Unicode | Verified (0.5 slice) | Unstarted | 0.5/0.8 | file URI + NFC/NFKC helpers |
| Logging | Verified (0.5 slice) | Unstarted | 0.5/0.8 | World handlers + optional `tracing` |
| Storage plug-ins | Dispositioned | Unstarted | 0.4/0.9 | legacy names → Unsupported; see `docs/design/0.4-legacy-storage.md` |
| `rdfproc` utility | Unstarted | n/a | 0.6 | CLI workflow inventory pending |

## Current evidence

- Inventory: curated 0.1–0.5 slices (0.5: 7 verified + 3 not-applicable).
- Integration tests cover world features/logging, CRUD, streams, SPARQL,
  storage, I/O, and utilities (`tests/utility.rs`).
- Utility digest smoke harness: `compatibility/harness/utility_digest_smoke.py`.
- Examples include `std_replacements` for hash/list migration.
- ADR-004–ADR-016 are accepted.
- Oxigraph 0.5.9 remains pinned with default features disabled.

## Next ledger upgrade

Generate the remaining Redland symbols from pinned headers once the broader
oracle harness expands. Next development focus: **0.6** safe Rust API parity
and `rdfproc`-equivalent workflows.
