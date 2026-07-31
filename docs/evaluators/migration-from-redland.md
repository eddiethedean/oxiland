# Migration from Redland

This page helps maintainers map Redland `librdf` workflows to Oxiland 0.4.
It is **not** a complete symbol-by-symbol porting guide (that is a 0.6
accounting goal). Inventories remain authoritative for claimed rows:

- [0.1 core inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.1.json)
- [0.2 I/O inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.2.json)
- [0.3 query inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.3.json)
- [0.5 storage/utilities inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.5.json)

## Mindset

1. Map **workflows**, not pointer ownership.
2. Expect typed `Result` and iterators instead of sentinel pointers.
3. Treat unknown factories/options as `Error::Unsupported`, not silent no-ops.
4. Check the [parity ledger](../parity.md) before asserting behavioral parity.

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

## C source and ABI

Not available. A separately audited `oxiland-capi` is planned no earlier than
0.8 ([ADR-002](../DECISIONS.md)). A Pythonic PyPI package is planned for 0.7 and
binds the safe Rust facade directly—not a mechanical port of every Rust
builder, and not layered on the C ABI. Do not schedule a binary drop-in C
migration on 0.4 timelines for storage; keep C ABI on 0.8+.

## Suggested migration sequence

1. Identify Redland workflows you actually call (parsers, model CRUD, SPARQL).
2. Confirm each is `verified` or `implemented` in the inventory for your
   milestone.
3. Port tests to Oxiland public APIs with differential fixtures where needed.
4. Keep native Redland as an oracle for contested behavior until fixtures pass.
5. Defer storage-plugin, Python package, and C ABI work until their milestones.
