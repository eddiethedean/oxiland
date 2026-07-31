# Positioning: Oxiland vs Oxigraph, Sophia, and Redland

## One-sentence summary

Oxiland is a **safe, Redland-shaped facade** on pinned Oxigraph—not a second RDF
engine, not a C drop-in today, and not a kitchen-sink Rust RDF toolkit.

## Compared to Oxigraph

| | Oxiland | Oxigraph |
|---|---|---|
| Role | Compatibility-oriented facade | RDF/SPARQL engine |
| API style | Redland concepts (Model, contexts, Syntax facades) | Native store / SPARQL / I/O types |
| Compatibility evidence | Inventories, parity ledger, milestone reports | Upstream standards tests |
| C ABI path | Planned separate crate (0.8+) | Not the product goal |
| Python path | Planned Pythonic PyPI package (0.7+) | Native Python APIs vary |
| When to pick | Migrating Redland *workflows* into Rust; want explicit unsupported errors | New apps that only need the engine |

Oxiland **depends on** Oxigraph. Choosing Oxiland always includes Oxigraph’s
semantics for standards machinery.

## Compared to Sophia

[Sophia](https://docs.rs/sophia) is a modular Rust RDF toolkit with its own
traits and ecosystem. Prefer Sophia when you want idiomatic Rust RDF abstractions
unrelated to Redland. Prefer Oxiland when Redland conceptual mapping and
inventory-backed claims matter.

## Compared to Redland (`librdf`)

| | Oxiland 0.4 | Native Redland |
|---|---|---|
| Language | Safe Rust | C (+ bindings) |
| Memory | Rust ownership | Manual / library conventions |
| I/O | Streaming facades for five syntaxes; transactional load | Raptor factories (broad) |
| SPARQL | Query/Update builders, streaming results, ResultsFormat | Rasqal (broader) |
| Storage | Supported Fjall format v1 + memory; transactions | Storage plugins |
| C consumers | Not yet | Yes |
| Drop-in ABI | No | N/A |

Oxiland targets measurable migration over time. It does **not** claim to replace
Redland in production C stacks in 0.4.

## What Oxiland optimizes for

1. Memory safety and data integrity
2. Honest, scoped compatibility claims
3. Standards-correct RDF/SPARQL via Oxigraph
4. A coherent Rust API

See the [charter](../CHARTER.md) for the full value order.
