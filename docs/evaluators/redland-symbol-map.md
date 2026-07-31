# Redland symbol map (0.6 accounting)

This page summarizes how public Redland `librdf` symbols map into Oxiland after
milestone 0.6. The authoritative list is the header-derived inventory:

[`compatibility/inventory/redland-1.0.17-oxiland-0.6.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.6.json)

Claim: **100% safe-API accounting** (every public function classified). This is
**not** C ABI parity.

## How to read the inventory

| State | Meaning |
|---|---|
| `verified` | Safe Rust mapping with tests |
| `not-applicable` | Replaced by Rust ownership / std collections |
| `excluded` | Out of scope; see `deviations` / `notes` |

## Subsystem cheat sheet

| Redland area | Oxiland |
|---|---|
| world / log / init | `World`, `LogLevel`, `LogFacility` |
| node / statement / model | `terms::*`, `Model`, `StatementPattern` |
| stream / iterator | fallible iterators (ADR-013) |
| parser / serializer | `io::Parser`, `io::Serializer`, `Syntax` |
| query / results | `Query`, `Update`, `QueryResults`, `ResultsFormat` |
| storage (memory / file-like) | `Model::new`, `Model::open` (Fjall) |
| storage plugins (MySQL, …) | `excluded` |
| digest / uri / utf8 | `utility::*` |
| hash / list | `not-applicable` → `HashMap` / `Vec` (ADR-016) |
| `librdf_new_*` / `librdf_free_*` | `not-applicable` → RAII |
| factory registration | `excluded` (ADR-018) |
| rdfproc CLI | `oxiland-cli` (ADR-019) |

## Workflow migration

Prefer [migration-from-redland.md](migration-from-redland.md) for end-to-end
workflows. Use this page + the inventory JSON when porting a specific symbol.
