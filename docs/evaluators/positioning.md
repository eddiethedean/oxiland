# Positioning: Oxiland vs Oxigraph, Sophia, and Redland

## One-sentence summary

Oxiland is an embedded RDF toolkit for Rust and Python, powered by a pinned
Oxigraph engine and differentiated by its compact facade, persistent-store
contract, CLI, and evidence-backed Redland migration path.

## Choose by requirement

| Requirement | Best starting point |
|---|---|
| A documented Rust facade with explicit storage/I/O/query contracts | Oxiland Rust crate |
| A typed Python wheel with embedded persistence and no Python runtime dependencies | Oxiland Python package |
| Direct access to the complete Oxigraph engine surface | Oxigraph |
| A trait-oriented modular Rust RDF ecosystem | Sophia |
| Existing `librdf` C ABI and storage plug-ins | Native Redland |
| Measured migration from Redland workflows | Oxiland migration and parity documentation |
| C source + librdf-compat packaging against a frozen allowlist | Oxiland `oxiland-capi` (build from source; see limitations) |

This is a boundary guide, not a universal benchmark. Validate syntax support,
store lifecycle, query behavior, target artifacts, and operational limits
against the application's own requirements.

## Compared to Oxigraph

| | Oxiland | Oxigraph |
|---|---|---|
| Role | Application-focused Rust/Python packages and CLI | RDF/SPARQL engine |
| API style | Redland concepts (Model, contexts, Syntax facades) | Native store / SPARQL / I/O types |
| Compatibility evidence | Inventories, parity ledger, milestone reports | Upstream standards tests |
| C path | Demonstrated 0.11 source/ABI matrix; see limitations for remaining gaps | Not the product goal |
| Python path | Typed PyPI package with its own API and production guide | Use the upstream Python surface when direct engine access is preferred |
| When to pick | The documented Oxiland facade, storage contract, CLI, or migration evidence adds value | Only the native engine API is required |

Oxiland **depends on** Oxigraph. Choosing Oxiland includes Oxigraph’s standards
machinery but not every upstream escape hatch in the Oxiland stability promise.

## Compared to Sophia

[Sophia](https://docs.rs/sophia) is a modular Rust RDF toolkit with its own
traits and ecosystem. Prefer Sophia when you want idiomatic Rust RDF abstractions
unrelated to Redland. Prefer Oxiland when Redland conceptual mapping and
inventory-backed claims matter.

## Compared to Redland (`librdf`)

| | Oxiland tip 0.12 | Native Redland |
|---|---|---|
| Language | Safe Rust + Python package + C ABI | C (+ bindings) |
| Memory | Rust ownership; opaque C handles | Manual / library conventions |
| I/O | Streaming facades for five syntaxes; transactional load | Raptor factories (broad) |
| SPARQL | Query/Update, streaming results, ResultsFormat | Rasqal (broader) |
| Storage | Memory/Fjall plus optional redb, RocksDB, SQLite, and LMDB; transactions | Storage plugins |
| CLI | `oxiland-cli` rdfproc-shaped (not binary drop-in) | `rdfproc` |
| Utilities / logging | Digests, URI helpers, vocab, World log handlers | librdf digests/logs/hashes |
| Safe-API accounting | Header-derived inventory classified (0.6+) | N/A |
| C consumers | Source-compat corpus + librdf-compat packaging on the frozen matrix | Yes |
| Drop-in ABI | Demonstrated on the verified 0.11 matrix; see C limitations | N/A |
| Performance claim | 0.12 competitive-parity gate closed (ADR-028); host-scoped wins after isolation — suite-wide faster-than-Redland via ADR-029 / qualify-0.13 (open until nine cells pass) | Baseline |

Oxiland targets measurable migration over time. Tip **0.11** demonstrates
Redland parity on the frozen matrix. Tip **0.12** closes the competitive-parity
performance gate; a suite-wide faster-than-Redland claim is gated by ADR-029
and `.github/workflows/qualify-0.13.yml` (three independent corrected-runner
passes per host).

## What Oxiland optimizes for

1. Memory safety and data integrity
2. Honest, scoped compatibility claims
3. Standards-correct RDF/SPARQL via Oxigraph
4. A coherent Rust (and Pythonic) API
5. Explicit production and failure semantics for supported workflows

Before adopting a persistent store, read the Rust or Python production guide.
Before making a compatibility claim, read the parity ledger and current report.
