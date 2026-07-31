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
| C source-compat preview against a frozen allowlist | Oxiland `oxiland-capi` (tip 0.8; build from source) |

This is a boundary guide, not a universal benchmark. Validate syntax support,
store lifecycle, query behavior, target artifacts, and operational limits
against the application's own requirements.

## Compared to Oxigraph

| | Oxiland | Oxigraph |
|---|---|---|
| Role | Application-focused Rust/Python packages and CLI | RDF/SPARQL engine |
| API style | Redland concepts (Model, contexts, Syntax facades) | Native store / SPARQL / I/O types |
| Compatibility evidence | Inventories, parity ledger, milestone reports | Upstream standards tests |
| C path | Source-compat preview crate in 0.8; ABI drop-in deferred to 0.9 | Not the product goal |
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

| | Oxiland 0.8 | Native Redland |
|---|---|---|
| Language | Safe Rust + Python package + C preview | C (+ bindings) |
| Memory | Rust ownership; opaque C handles in the preview | Manual / library conventions |
| I/O | Streaming facades for five syntaxes; transactional load | Raptor factories (broad) |
| SPARQL | Query/Update, streaming results, ResultsFormat | Rasqal (broader) |
| Storage | Supported Fjall format v1 + memory; transactions | Storage plugins |
| CLI | `oxiland-cli` rdfproc-shaped (not binary drop-in) | `rdfproc` |
| Utilities / logging | Digests, URI helpers, vocab, World log handlers | librdf digests/logs/hashes |
| Safe-API accounting | Header-derived inventory classified (0.6+) | N/A |
| C consumers | Source-compat preview (0.8 allowlist); not ABI drop-in | Yes |
| Drop-in ABI | No (planned 0.9 evidence) | N/A |

Oxiland targets measurable migration over time. Tip **0.8** offers an auditable
C source-compat preview for a frozen allowlist; it does **not** claim to replace
Redland in production C stacks as an ABI drop-in.

## What Oxiland optimizes for

1. Memory safety and data integrity
2. Honest, scoped compatibility claims
3. Standards-correct RDF/SPARQL via Oxigraph
4. A coherent Rust (and Pythonic) API
5. Explicit production and failure semantics for supported workflows

Before adopting a persistent store, read the Rust or Python production guide.
Before making a compatibility claim, read the parity ledger and current report.
