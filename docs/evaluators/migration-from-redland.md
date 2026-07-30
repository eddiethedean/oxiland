# Migration from Redland

This page helps maintainers map Redland `librdf` workflows to Oxiland 0.2.
It is **not** a complete symbol-by-symbol porting guide (that is a 0.6
accounting goal). Inventories remain authoritative for claimed rows:

- [0.1 core inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.1.json)
- [0.2 I/O inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.2.json)

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
| Storage plugins | Not mapped; Fjall via `Model::open` is experimental |

## Parser / serializer (0.2)

| Redland concept | Oxiland |
|---|---|
| `librdf_new_parser` / name / MIME | `Syntax::from_name` / `from_media_type` → `Parser::for_syntax` |
| Parse as stream | `Parser::parse_reader` / `parse_str` / `parse_path` |
| Parse into model | `load_into` (progressive) or `load_collecting` |
| Guess / sniff | Unsupported — explicit `Syntax` or extension API |
| `librdf_new_serializer` | `Serializer::for_syntax` |
| Namespaces | `Serializer::with_prefix` (Turtle/TriG/RDF/XML only) |
| Serialize model | `serialize_model_to_string` / `_to_path` / `_to_writer` |
| N3 / JSON-LD factories | Unsupported or deferred |

Design detail: [docs/design/0.2-io-api.md](../design/0.2-io-api.md).

## Query

| Redland concept | Oxiland 0.2 |
|---|---|
| Create/execute SPARQL | `Query::new(...).execute(&model)` |
| ASK / SELECT | Supported at a basic level |
| Update / rich results | Planned 0.3 |

## C source and ABI

Not available. A separately audited `oxiland-capi` is planned no earlier than
0.7 ([ADR-002](../DECISIONS.md)). Do not schedule a binary drop-in migration on
0.2 timelines.

## Suggested migration sequence

1. Identify Redland workflows you actually call (parsers, model CRUD, SPARQL).
2. Confirm each is `verified` or `implemented` in the inventory for your
   milestone.
3. Port tests to Oxiland public APIs with differential fixtures where needed.
4. Keep native Redland as an oracle for contested behavior until fixtures pass.
5. Defer storage-plugin and C ABI work until their milestones.
