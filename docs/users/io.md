# RDF input and output

Oxiland 0.2 provides Redland-shaped [`Parser`](https://docs.rs/oxiland/latest/oxiland/io/struct.Parser.html)
and [`Serializer`](https://docs.rs/oxiland/latest/oxiland/io/struct.Serializer.html)
facades over a closed [`Syntax`](https://docs.rs/oxiland/latest/oxiland/io/enum.Syntax.html)
set (ADR-008).

## Choosing a syntax

```rust
use oxiland::io::Syntax;

let turtle = Syntax::from_name("turtle")?;
let same = Syntax::from_media_type("text/turtle; charset=utf-8")?;
let from_ext = Syntax::from_extension("ttl")?;
assert_eq!(turtle, same);
assert_eq!(turtle, from_ext);
# Ok::<(), oxiland::Error>(())
```

Unknown names, N3, JSON-LD, content sniffing (`guess`), and ambiguous aliases
such as `text/plain` or `.xml` return `Error::Unsupported`.

| Syntax | Datasets (named graphs) |
|---|---|
| Turtle, N-Triples, RDF/XML | No |
| N-Quads, TriG | Yes |

## Graph targets

| Target | Behavior |
|---|---|
| `GraphTarget::DefaultGraph` (default) | Emit default-graph quads; **reject** named-graph input |
| `GraphTarget::Named(g)` | Remap the syntax default graph into `g`; reject other named graphs |
| `GraphTarget::Dataset` | Preserve TriG/N-Quads named graphs; unsupported for graph-only syntaxes |

## Progressive vs collecting load (ADR-007)

```text
load_into        → insert as you parse; partial data may remain on failure
load_collecting  → parse fully first; insert only after a successful parse
                   (rolls back quads this call newly inserted if insert fails)
```

Progressive load is honest about partial progress. On Fjall-backed models each
successful insert is durable, so a failed progressive load can leave data on
disk. Prefer `load_collecting` when you need parse-then-insert batching without
transactions (true transactional import is planned for 0.4).

Demo: `cargo run --example progressive_load`.

## Streaming

`Parser::parse_reader` / `parse_slice` yield `Result<Quad>` iterators. Stop
early by dropping the iterator—do not collect the whole document unless you
intend to.

`Serializer::serialize_model_to_writer` streams from `Model::find` and does not
buffer a second full copy of the dataset.

## Errors

| Category | Typical cause |
|---|---|
| `Error::Parse` | Malformed RDF (optional `SourceLocation`) |
| `Error::Io` | Reader/writer/filesystem failure |
| `Error::Serialize` | RDF serialize configuration / invalid input for format |
| `Error::Unsupported` | Bad syntax name, GraphTarget, or graph-only vs dataset mismatch |
| `Error::InvalidRdf` | Bad base IRI or prefix IRI at configuration time |

## See also

- Design notes: [docs/design/0.2-io-api.md](../design/0.2-io-api.md)
- Format dispositions: [format-matrix.json](https://github.com/eddiethedean/oxiland/blob/main/compatibility/baseline/format-matrix.json)
- Redland I/O mapping: [migration guide](../evaluators/migration-from-redland.md)
