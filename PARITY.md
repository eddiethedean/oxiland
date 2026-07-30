# Redland parity ledger

Current milestone: 0.1 (complete)  
Ledger maturity: curated 0.1 inventory slice; full header-derived generation
pending with the oracle harness

Target: the documented Redland `librdf` 1.0.17 API (manual labeled 1.0.18).

Planned sequencing and completion rules are documented in the
[0.x roadmap](docs/ROADMAP.md) and
[compatibility plan](docs/COMPATIBILITY.md).

Inventory revision:
[`compatibility/inventory/redland-1.0.17-oxiland-0.1.json`](compatibility/inventory/redland-1.0.17-oxiland-0.1.json)

0.1 compatibility report: [`docs/reports/0.1.md`](docs/reports/0.1.md)

## Status vocabulary

- `unreviewed`: not yet mapped from canonical Redland inputs.
- `mapped`: intended Rust/C representation is documented.
- `implemented`: code exists but lacks complete compatibility evidence.
- `verified`: required evidence passes for the stated scope.
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
| Storage | Partial | Unstarted | 0.4/0.8 | memory default; redb persistence via `Model::open` |
| Streams / iterators | Verified (find) | Unstarted | 0.5/0.7 | `StatementMatches` streaming iterator (ADR-005) |
| Parser | Primitive only | Unstarted | 0.2/0.7 | Oxigraph primitive re-export; facade pending |
| Serializer | Primitive only | Unstarted | 0.2/0.7 | Oxigraph primitive re-export; facade pending |
| SPARQL query/results | Partial | Unstarted | 0.3/0.7 | ASK/SELECT execution; parse vs evaluation errors |
| Query update | Unstarted | Unstarted | 0.3/0.7 | Oxigraph capability not yet exposed |
| Digests | Unstarted | Unstarted | 0.5/0.7 | inventory and mapping pending |
| Hashes / lists | Unreviewed | Unstarted | 0.5/0.7 | likely Rust replacements; rationale required |
| Heuristics / files / Unicode | Unstarted | Unstarted | 0.5/0.7 | inventory and mapping pending |
| Logging | Unstarted | Unstarted | 0.5/0.7 | callback and `tracing` design pending |
| Storage plug-ins | Unreviewed | Unstarted | 0.4/0.8 | per-backend decisions required |
| `rdfproc` utility | Unstarted | n/a | 0.6 | CLI workflow inventory pending |

## Current evidence

- Inventory: 22 curated 0.1 entries (18 verified, 4 implemented).
- Integration tests cover world features, default CRUD, named-graph isolation,
  streaming find, SPARQL ASK/SELECT, invalid IRI/blank-node input, SPARQL
  parse errors, and unsupported storage backends.
- Doctests cover construction, CRUD, and SPARQL on public types.
- Examples `quick_start` and `contexts` run in CI.
- CI gates: Check + MSRV (1.87) on every PR and `main`/release.
- Oxigraph 0.5.9 is pinned with default features disabled.
- ADR-004 (term re-exports) and ADR-005 (streaming find) are accepted.

This evidence validates the 0.1 safe-core claim. It is not differential
evidence against native Redland.

## Next ledger upgrade

Generate the remaining Redland symbols from pinned headers once the oracle
harness lands. Expand verified rows only when differential or standards
fixtures exist for the claimed behavior.

“100% parity” is reached only when every public Redland function is represented
in a generated symbol inventory, has a documented mapping or intentional
safe-Rust replacement, and satisfies the evidence required for the specific
compatibility claim. No blended percentage is used.
