# Redland parity ledger

Current milestone: 0.1  
Ledger maturity: subsystem bootstrap; symbol-level inventory not generated yet

Target: the documented Redland `librdf` 1.0.17 API (manual labeled 1.0.18).

Planned sequencing and completion rules are documented in the
[0.x roadmap](docs/ROADMAP.md) and
[compatibility plan](docs/COMPATIBILITY.md).

## Status vocabulary

- `unreviewed`: not yet mapped from canonical Redland inputs.
- `mapped`: intended Rust/C representation is documented.
- `implemented`: code exists but lacks complete compatibility evidence.
- `verified`: required evidence passes for the stated scope.
- `not-applicable`: legacy mechanism replaced safely with rationale.
- `excluded`: intentionally unsupported with an approved impact assessment.

`Partial` below is a temporary subsystem summary, not an inventory state.

## Subsystem summary

| Redland subsystem | Safe Rust | C ABI | Target | Current evidence / gap |
|---|---|---|---:|---|
| World / lifecycle | Partial | Unstarted | 0.1/0.7 | RAII world and feature registry; factories/logging pending |
| URI | Partial | Unstarted | 0.1/0.7 | validated Oxigraph named nodes; helper parity unreviewed |
| Nodes | Partial | Unstarted | 0.1/0.7 | Oxigraph term exports; wrapper decision open |
| Statements | Partial | Unstarted | 0.1/0.7 | triples and partial matching; lifecycle inventory pending |
| Model | Partial | Unstarted | 0.1/0.7 | default CRUD, size, patterns, contexts; context CRUD gaps |
| Storage | Partial | Unstarted | 0.4/0.8 | memory implemented; optional RocksDB constructor |
| Streams / iterators | Partial | Unstarted | 0.5/0.7 | current matching materializes a `Vec` |
| Parser | Primitive only | Unstarted | 0.2/0.7 | Oxigraph primitive re-export; facade pending |
| Serializer | Primitive only | Unstarted | 0.2/0.7 | Oxigraph primitive re-export; facade pending |
| SPARQL query/results | Partial | Unstarted | 0.3/0.7 | basic execution; full result/configuration surface pending |
| Query update | Unstarted | Unstarted | 0.3/0.7 | Oxigraph capability not yet exposed |
| Digests | Unstarted | Unstarted | 0.5/0.7 | inventory and mapping pending |
| Hashes / lists | Unreviewed | Unstarted | 0.5/0.7 | likely Rust replacements; rationale required |
| Heuristics / files / Unicode | Unstarted | Unstarted | 0.5/0.7 | inventory and mapping pending |
| Logging | Unstarted | Unstarted | 0.5/0.7 | callback and `tracing` design pending |
| Storage plug-ins | Unreviewed | Unstarted | 0.4/0.8 | per-backend decisions required |
| `rdfproc` utility | Unstarted | n/a | 0.6 | CLI workflow inventory pending |

## Current evidence

- Three integration tests cover world construction, model CRUD/pattern
  matching, named-graph insertion, and SPARQL `ASK`.
- `cargo test`, Clippy with warnings denied, and Rustdoc pass locally.
- Oxigraph 0.5.9 is pinned with default features disabled.

This evidence validates only the named workflows. It is not yet differential
evidence against Redland.

## Next ledger upgrade

The 0.1 P0 task is to replace this subsystem bootstrap with a generated,
versioned symbol/type/enum inventory. Each row will carry stable IDs,
implementation locations, fixture IDs, state, platform/features, and evidence
revisions as specified by the [compatibility plan](docs/COMPATIBILITY.md).

“100% parity” is reached only when every public Redland function is represented
in a generated symbol inventory, has a documented mapping or intentional
safe-Rust replacement, and satisfies the evidence required for the specific
compatibility claim. No blended percentage is used.
