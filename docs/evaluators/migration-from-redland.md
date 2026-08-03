# Migration from Redland

This page helps maintainers map Redland `librdf` workflows to **Oxiland tip
0.11** (safe Rust facade + `oxiland-cli` + PyPI package + C ABI on the verified
matrix). Milestone 0.11 closed the demonstrated full-parity gate;
[milestone 0.12](../milestones/0.12.md) is the planned performance-optimization
phase before 1.0. For **symbol-by-symbol** accounting see
[redland-symbol-map.md](redland-symbol-map.md), the header-derived safe-API
inventory
[`redland-1.0.17-oxiland-0.6.json`](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.6.json),
and the [0.11 inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.11.json)
(Python usability evidence is in the [0.7 report](../reports/0.7.md), not a
second `librdf` inventory).

Earlier curated slices remain available for historical milestone evidence:

- [0.1 core inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.1.json)
- [0.2 I/O inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.2.json)
- [0.3 query inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.3.json)
- [0.4 storage inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.4.json)
- [0.5 streams/utilities inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.5.json)

## Mindset

1. Map **workflows**, not pointer ownership.
2. Expect typed `Result` and iterators instead of sentinel pointers.
3. Treat unknown factories/options as `Error::Unsupported`, not silent no-ops.
4. Check the [parity ledger](../parity.md) before asserting behavioral parity.

## Assess before porting

Inventory the application rather than translating headers wholesale:

| Question | Why it matters |
|---|---|
| Which parsers, query forms, storage backends, and feature URIs are used? | Determines verified mappings and explicit exclusions |
| Does the application require C source or ABI compatibility? | Tip 0.11 demonstrates the frozen matrix; check [C limitations](../users/c-abi-limitations.md) for remaining gaps |
| Which data must survive an upgrade? | Requires N-Quads export, restore rehearsal, and format planning |
| Are inputs or queries untrusted? | Requires application budgets and isolation beyond library semantics |
| Which errors and callback orders affect control flow? | Must be covered by differential fixtures, not assumed from successful cases |
| Can the migration be rolled back? | Determines dual-run, backup, and cutover design |

## Core model (0.1)

| Redland concept | Oxiland |
|---|---|
| `librdf_world` | `World` (feature registry; RAII) |
| URI / nodes | `terms::NamedNode`, `BlankNode`, `Literal`, helpers |
| Statement | `terms::Triple` / `Quad` |
| Model CRUD | `Model::add`, `remove`, `contains`, `len` |
| Contexts | `add_to_graph` / `GraphName` |
| Find / streams | `Model::find` → `StatementMatches` |
| Storage “memory” | `Model::new` |
| Storage plugins | Not 1:1; Fjall via `Model::open` / `OpenOptions` is the supported durable backend (format v1) |

## Parser / serializer (0.2)

| Redland concept | Oxiland |
|---|---|
| `librdf_new_parser` / name / MIME | `Syntax::from_name` / `from_media_type` → `Parser::for_syntax` |
| Parse as stream | `Parser::parse_reader` / `parse_str` / `parse_path` |
| Parse into model | `load_into` (progressive), `load_collecting`, or `load_transactional` (0.4) |
| Guess / sniff | Unsupported — explicit `Syntax` or extension API |
| `librdf_new_serializer` | `Serializer::for_syntax` |
| Namespaces | `Serializer::with_prefix` (Turtle/TriG/RDF/XML only) |
| Serialize model | `serialize_model_to_string` / `_to_path` / `_to_writer` |
| N3 / JSON-LD factories | Unsupported or deferred |

Design detail: [docs/design/0.2-io-api.md](../design/0.2-io-api.md).

## Query / update / results (0.3)

| Redland concept | Oxiland |
|---|---|
| Create/execute SPARQL | `Query::new(...).execute(&model)` |
| ASK / SELECT / CONSTRUCT / DESCRIBE | Streaming `QueryResults` |
| Limit / offset / dataset | `Query::limit` / `offset` / `default_graph` / … |
| SPARQL Update | `Update::new(...).execute(&model)` |
| Results to string | `ResultsFormat` + `serialize_query_results_to_string` |
| Graph results to RDF | `serialize_graph_results_to_writer` / `io::Serializer` |
| Cancel | `CancellationToken` (wall-clock timeout is caller-driven) |

Design detail: [docs/design/0.3-query-api.md](../design/0.3-query-api.md).

## Utilities, digests, logging (0.5)

| Redland concept | Oxiland |
|---|---|
| Digests (MD5/SHA) | `utility::DigestAlgorithm`, `digest_hex` / `digest_path` |
| URI join / file URI | `utility::join_iri`, `path_to_file_uri`, `file_uri_to_path` |
| Unicode normalize | `utility::normalize_nfc` / `normalize_nfkc` |
| Namespaces / vocab IRIs | `utility::Namespace`, `utility::vocab::{rdf,rdfs,xsd,owl,dc}` |
| World logging | `World::set_log_handler`, `LogLevel`, `LogFacility` (optional feature `tracing`) |
| `librdf_hash` | **not-applicable** — use `std::collections::HashMap` (ADR-016) |
| `librdf_list` | **not-applicable** — use `Vec` / iterators (ADR-016) |
| `librdf_free_*` | **not-applicable** — Rust ownership / `Drop` |

Demo: `cargo run --example std_replacements`.

## CLI / rdfproc (0.6)

| Redland concept | Oxiland |
|---|---|
| `rdfproc` parse / find / query / serialize | `oxiland-cli` (ADR-019) |
| storage `memory` | `-s memory` |
| storage file path | `-s fjall` path with `-n` to create (format v1) |
| MySQL/Virtuoso/`hashes` plugins | Unsupported (explicit error) |

See [cli.md](../users/cli.md) and [redland-symbol-map.md](redland-symbol-map.md).

## Python (0.7)

| Redland / Python concept | Oxiland |
|---|---|
| Native Redland Python bindings | **Not** a drop-in; use `pip install oxiland` |
| Model CRUD / contexts | `Model`, `Triple`/`Quad`, `find` |
| Parse / serialize | `load`, `serialize`, `parse` |
| SPARQL | `query`, `update`, `serialize_results` |
| rdflib interop | Deferred (ADR-017) |

Guide: [python.md](../users/python.md).

## C source and ABI

A separately audited `oxiland-capi` **source-compat preview** is available in
tip **0.10**. Build it
from this repository—it is not published on crates.io. It is **not** a binary
or ABI drop-in for existing Redland shared libraries; only symbols in its
shipped header are exported. See the [C ABI guide](../users/c-abi.md) and
[limitations](../users/c-abi-limitations.md). Inventory rows for the preview
live in the
[0.9 inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.9.json)
(`c_abi` / `c_state`).

A Pythonic PyPI package ships independently (`pip install oxiland`) and binds
the safe Rust facade directly—not a mechanical port of every Rust builder, and
not layered on the C ABI. Prefer Rust/Python migration when ABI drop-in is not
required; do not schedule binary ABI replacement until the 0.10 report passes.

## Suggested migration sequence

1. Identify Redland workflows you actually call (parsers, model CRUD, SPARQL).
2. Confirm each is `verified`, `not-applicable`, or `excluded` in the current
   inventory (0.6 safe-API accounting; 0.8 adds `c_abi` / `c_state` for the
   preview allowlist; the 0.9 revision records the latest completed C accounting).
3. Port tests to Oxiland public APIs with differential fixtures where needed.
4. Keep native Redland as an oracle for contested behavior until fixtures pass.
5. Prefer the PyPI package for Python callers; evaluate the C preview against
   the allowlist only, and keep ABI drop-in expectations blocked on the 0.10
   qualification report.

## Production cutover

1. Export the source dataset in a standards format that preserves named graphs.
2. Import into an isolated Oxiland store and record processed counts.
3. Run application-owned queries plus the relevant differential fixtures.
4. Compare failure behavior as well as successful output.
5. Capacity-test memory, disk, and representative query/update latency.
6. Define one store owner, backup retention, restore rehearsal, and rollback.
7. Cut over only after the published compatibility scope matches the actual
   workflows in use.

Use the [Rust](../users/rust-production.md) or
[Python](../users/python-production.md) production runbook for the target
package.
