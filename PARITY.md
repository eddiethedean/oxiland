# Redland parity ledger

Last completed milestone: 0.12
Current development milestone: 1.0 readiness (`planned` — after complete 0.11
parity and 0.12 competitive-parity performance gate; see `docs/ROADMAP.md`)
Ledger maturity: header-derived full public `librdf` 1.0.17 function inventory
(0.6); Python package usability evidence (0.7); C ABI source-compat accounting
(0.8–0.9); curated 0.1–0.5 slices retained for historical evidence;
0.10 candidate full-parity inventory and qualification scaffold; 0.11
demonstrated parity from six-cell native differentials (`scripts/check-0.11-release.py`
green on revision-bound raw evidence); 0.12 closed the ADR-028 competitive-parity
performance gate on that parity baseline (suite-wide faster-than-Redland remains
open)

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
- [`redland-1.0.17-oxiland-0.11.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.11.json)

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
| World / lifecycle | Verified (0.11) | Verified (0.11) | 0.11 | RAII world; C world_open differentials |
| URI | Verified (0.11) | Verified (0.11) | 0.11 | join/relativize/file-URI; C URI create/free |
| Nodes | Verified (0.11) | Verified (0.11) | 0.11 | Oxigraph terms; C URI/literal nodes |
| Statements | Verified (0.11) | Verified (0.11) | 0.11 | triples/`StatementPattern`; C statement handles |
| Model | Verified (0.11) | Verified (0.11) | 0.11 | CRUD + find; C model differentials |
| Storage | Verified (0.11) | Verified (0.11) | 0.11 | DurableStore; C memory/fjall + optional backends |
| Streams / iterators | Verified (0.11) | Verified (0.11) | 0.11 | find/parse/query streams; C stream allowlist |
| Parser | Verified (0.11) | Verified (0.11) | 0.11 | `Parser` facade; C parse differentials |
| Serializer | Verified (0.11) | Verified (0.11) | 0.11 | `Serializer` facade; C serialize differentials |
| SPARQL query/results | Verified (0.11) | Verified (0.11) | 0.11 | ASK/SELECT/CONSTRUCT/DESCRIBE C path |
| Query update | Verified (0.11) | Verified (0.11) | 0.11 | `Update` facade; C update in 0.11 inventory |
| Digests | Verified (0.11) | Verified (0.11) | 0.11 | MD5/SHA-1/SHA-256 (ADR-015) |
| Hashes / lists | Dispositioned | Verified (0.11) | 0.11 | Rust N/A → `HashMap`/`Vec`; C list/hash surface |
| Heuristics / files / Unicode | Verified (0.11) | Verified (0.11) | 0.11 | file URI + NFC/NFKC; C helpers |
| Logging | Verified (0.11) | Verified (0.11) | 0.11 | World handlers + `librdf_log_simple` path |
| Storage plug-ins | Dispositioned | Verified (0.11) | 0.11 | baseline factories; out-of-baseline plugins excluded |
| `rdfproc` utility | Verified (0.11) | n/a | 0.11 | `oxiland-cli` + harness CLI fixtures |
| Python / PyPI package | Verified (0.7 usability) | n/a | 0.7 | `pip install oxiland`; wheels + pytest + typing |

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
three synthetic performance profiles, a soak record, and fuzz-smoke records are
checked in.
`scripts/check-0.10-release.py` passes. This does not establish full Redland
parity: the profile pass declarations were generated from shared symbol
presence and local smoke suites rather than native two-sided executions on each
declared target. The performance fixtures likewise validate the report tooling
but do not establish native faster-than-Redland results.
See the
[`0.10 qualification report`](https://github.com/eddiethedean/oxiland/blob/main/docs/reports/0.10.md).
Inventory revision:
[`redland-1.0.17-oxiland-0.10.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.10.json).

## Current 0.11 qualification

Complete. Six target/profile cells use **C-oracle** differentials (system
librdf ↔ Oxiland librdf-compat), with failure/boundary/lifecycle fixtures,
independent native performance benches (`synthetic: false`), ABI-swap
evidence, and `scripts/check-0.11-release.py` green on the revision-bound tip.
See [`docs/reports/0.11.md`](https://github.com/eddiethedean/oxiland/blob/main/docs/reports/0.11.md).

## Completed 0.12 performance qualification

Complete for its stated ADR-028 competitive-parity gate on the committed
three-host bundle (`scripts/check-0.12-release.py` green). Tip retains 0.11
parity on the optimized candidate. Host-scoped strict wins after library-path
isolation are documented in the performance guide; a suite-wide
faster-than-Redland claim still requires three independent corrected-runner
passes per host.
See [`docs/reports/0.12.md`](https://github.com/eddiethedean/oxiland/blob/main/docs/reports/0.12.md)
and [`docs/users/performance.md`](https://github.com/eddiethedean/oxiland/blob/main/docs/users/performance.md).
