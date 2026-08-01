# Redland parity ledger

Last completed milestone: 0.10
Current development milestone: 0.11 (`in progress`)
Ledger maturity: header-derived full public `librdf` 1.0.17 function inventory
(0.6); Python package usability evidence (0.7); C ABI source-compat preview
accounting (0.8); curated 0.1–0.5 slices retained for historical evidence;
0.10 candidate full-parity inventory and qualification scaffold; 0.11 native
differential, source-compatibility, and binary-interchange qualification active

> **Newcomer gloss:** This ledger classifies Redland `librdf_*` symbols and
> records what Oxiland has implemented with tests for a **stated scope**.
> **“100% safe-API accounting” means every inventoried symbol is classified**
> (`verified`, `not-applicable`, or `excluded`)—**not** that Oxiland is a
> drop-in Redland replacement or that every Redland behavior has a differential
> test. Prefer Oxigraph directly if you do not need Redland-shaped APIs.

Target: the documented Redland `librdf` 1.0.17 API (manual labeled 1.0.18).

Planned sequencing and completion rules are documented in the
[0.x roadmap](https://github.com/eddiethedean/oxiland/blob/main/docs/ROADMAP.md) and
[compatibility plan](https://github.com/eddiethedean/oxiland/blob/main/docs/COMPATIBILITY.md).

Inventory revisions:

- [`redland-1.0.17-oxiland-0.1.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.1.json)
- [`redland-1.0.17-oxiland-0.2.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.2.json)
- [`redland-1.0.17-oxiland-0.3.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.3.json)
- [`redland-1.0.17-oxiland-0.4.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.4.json)
- [`redland-1.0.17-oxiland-0.5.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.5.json)
- [`redland-1.0.17-oxiland-0.6.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.6.json)
- [`redland-1.0.17-oxiland-0.8.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.8.json)
- [`redland-1.0.17-oxiland-0.9.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.9.json)
- [`redland-1.0.17-oxiland-0.10.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.10.json)

0.6 compatibility report: [`docs/reports/0.6.md`](https://github.com/eddiethedean/oxiland/blob/main/docs/reports/0.6.md)
· 0.7 report: [`docs/reports/0.7.md`](https://github.com/eddiethedean/oxiland/blob/main/docs/reports/0.7.md)
· 0.8 report: [`docs/reports/0.8.md`](https://github.com/eddiethedean/oxiland/blob/main/docs/reports/0.8.md)

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
| World / lifecycle | Verified (0.6 accounting) | Verified (preview) | 0.1/0.8 | RAII world; C allowlist world_open (ADR-014/023) |
| URI | Verified (0.6 accounting) | Verified (preview) | 0.5/0.8 | join/relativize/file-URI helpers; C URI create/free |
| Nodes | Verified (0.6 accounting) | Verified (preview) | 0.1/0.8 | Oxigraph term re-exports; C URI/literal nodes |
| Statements | Verified (0.6 accounting) | Verified (preview) | 0.1/0.8 | triples and `StatementPattern`; C statement handles |
| Model | Verified (0.6 accounting) | Verified (preview) | 0.1/0.8 | CRUD + find; C model allowlist |
| Storage | Verified (0.6 accounting) | Verified (preview) | 0.4/0.9 | sealed DurableStore; C memory/fjall; optional backends 0.9 |
| Streams / iterators | Verified (0.6 accounting) | Verified (preview) | 0.5/0.8 | find/parse/query streams; C stream allowlist |
| Parser | Verified (0.6 accounting) | Verified (preview) | 0.2/0.8 | `Parser` facade; C parse-string allowlist |
| Serializer | Verified (0.6 accounting) | Verified (preview) | 0.2/0.8 | `Serializer` facade; C serialize-to-string allowlist |
| SPARQL query/results | Verified (0.6 accounting) | Verified (preview) | 0.3/0.8 | ASK/SELECT on C; CONSTRUCT/DESCRIBE C deferred |
| Query update | Verified (0.6 accounting) | Unstarted | 0.3/0.9 | `Update` facade; C update not in 0.8 allowlist |
| Digests | Verified (0.6 accounting) | Unstarted | 0.5/0.9 | MD5/SHA-1/SHA-256 (ADR-015); C deferred |
| Hashes / lists | Dispositioned | Unstarted | 0.5/0.8 | `not-applicable` → `HashMap`/`Vec` (ADR-016) |
| Heuristics / files / Unicode | Verified (0.6 accounting) | Unstarted | 0.5/0.9 | file URI + NFC/NFKC helpers; C deferred |
| Logging | Verified (0.6 accounting) | Unstarted | 0.5/0.9 | World handlers + optional `tracing`; C log handlers deferred |
| Storage plug-ins | Dispositioned | Unstarted | 0.4/0.9 | excluded / Unsupported |
| `rdfproc` utility | Verified (0.6 CLI) | n/a | 0.6 | `oxiland-cli` workflows (ADR-019) |
| Python / PyPI package | Verified (0.7 usability) | n/a | 0.7 | `pip install oxiland`; wheels + pytest + typing (ADR-017) |

## Safe-API accounting (0.6)

**100% safe-API accounting** means every header-derived public `librdf_*`
function in the 0.6 inventory is **classified** (not that behavior is a Redland
drop-in): 383 classified (238 verified, 96 not-applicable, 49 excluded); 0
unreviewed. See [`docs/reports/0.6.md`](https://github.com/eddiethedean/oxiland/blob/main/docs/reports/0.6.md).

## C ABI accounting (0.8 preview → 0.9)

0.9 inventory closes C gaps: allowlist symbols are `c_state=verified`; remaining
applicable symbols are justified `not-applicable` / `excluded` with notes.
Source-compat plus measured Oxiland ABI—**not** Redland binary `.so` drop-in.
See [`docs/reports/0.9.md`](https://github.com/eddiethedean/oxiland/blob/main/docs/reports/0.9.md) and
[`docs/reports/0.8.md`](https://github.com/eddiethedean/oxiland/blob/main/docs/reports/0.8.md).

## Current evidence

- Inventory: full header-derived manifests through 0.9 C-field revision plus
  curated earlier slices.
- Integration tests + `tests/accounting.rs` + `oxiland-cli` tests +
  `tests/backend_conformance.rs`.
- CLI smoke: `compatibility/harness/cli_smoke.py`.
- Python: `python/tests/`, pyright, wheel smoke in CI; report
  [`docs/reports/0.7.md`](https://github.com/eddiethedean/oxiland/blob/main/docs/reports/0.7.md).
- C ABI: `crates/oxiland-capi` example, symbol allowlist, ASan CI; report
  [`docs/reports/0.8.md`](https://github.com/eddiethedean/oxiland/blob/main/docs/reports/0.8.md).
- Public API snapshot + `cargo semver-checks` vs 0.9.0.
- ADR-004–ADR-023 accepted as applicable.
- Oxigraph 0.5.9 remains pinned with default features disabled.

## Completed 0.10 qualification scaffold

Complete for its stated scaffold. Inventory, declared six-profile parity data,
three performance profiles, soak, and fuzz records are checked in.
`scripts/check-0.10-release.py` passes. This does not establish full Redland
parity: the profile pass declarations were generated from shared symbol
presence and local smoke suites rather than native two-sided executions on each
declared target.
See the
[`0.10 qualification report`](https://github.com/eddiethedean/oxiland/blob/main/docs/reports/0.10.md).
Inventory revision:
[`redland-1.0.17-oxiland-0.10.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.10.json).

## Current 0.11 qualification

In progress. Milestone 0.11 re-verifies every inherited claim from raw native
Redland and Oxiland executions, expands the denominator beyond function names,
and requires unchanged-source C builds plus Redland-built binaries running
against Oxiland without rebuild or relink. Evidence must be produced separately
on each supported target/profile and bound to the exact clean revision,
fixtures, harnesses, and artifacts.

No 0.11 full-parity claim exists until the fail-closed gate described by the
[`0.11 milestone`](https://github.com/eddiethedean/oxiland/blob/main/docs/milestones/0.11.md)
passes from raw evidence. Current gaps are tracked in the
[`0.11 report`](https://github.com/eddiethedean/oxiland/blob/main/docs/reports/0.11.md).
