# Rust RDF input and output

`oxiland::io` provides configured parsers and serializers for files, byte
slices, strings, arbitrary readers, arbitrary writers, and complete models.
Format selection and graph handling are explicit so unsupported or ambiguous
input cannot silently change dataset semantics.

## Supported syntaxes

| Syntax | Name | Media type | Extension | Named graphs |
|---|---|---|---|---|
| Turtle | `turtle` | `text/turtle` | `.ttl` | No |
| N-Triples | `ntriples` | `application/n-triples` | `.nt` | No |
| N-Quads | `nquads` | `application/n-quads` | `.nq` | Yes |
| TriG | `trig` | `application/trig` | `.trig` | Yes |
| RDF/XML | `rdfxml` | `application/rdf+xml` | `.rdf` | No |

```rust
use oxiland::io::Syntax;

assert_eq!(Syntax::from_name("turtle")?, Syntax::Turtle);
assert_eq!(Syntax::from_media_type("text/turtle; charset=utf-8")?, Syntax::Turtle);
assert_eq!(Syntax::from_extension(".ttl")?, Syntax::Turtle);
# Ok::<(), oxiland::Error>(())
```

Unknown names, N3, JSON-LD, content sniffing, and ambiguous aliases such as
`text/plain`, `.txt`, or `.xml` return `Error::Unsupported`. The name `xml` is
accepted as an RDF/XML alias, but extension lookup requires `.rdf` or `.owl`.

## Stream RDF

```rust
use oxiland::io::{GraphTarget, Parser, Syntax};

# fn main() -> oxiland::Result<()> {
let parser = Parser::for_syntax(Syntax::Turtle)
    .base_iri("https://example.com/")?
    .graph_target(GraphTarget::DefaultGraph);

for item in parser.parse_str("<alice> <name> \"Alice\" .")? {
    let quad = item?;
    println!("{quad}");
}
# Ok(())
# }
```

`parse_reader`, `parse_slice`, `parse_str`, and `parse_path` return lazy,
fallible quad iterators. Parser creation validates configuration; syntax and
mid-stream I/O errors can occur while advancing the iterator. A UTF-8 BOM is
stripped when present.

Use `Parser::parse_path_with_extension(path)` when extension selection is
desired. It returns `(Syntax, stream)` and automatically chooses dataset mode
for `.nq` and `.trig`.

## Graph targets

Every parser has an explicit destination policy:

| Target | Behavior |
|---|---|
| `GraphTarget::DefaultGraph` | Emit default-graph quads and reject named-graph input |
| `GraphTarget::Named(graph)` | Remap syntax-default triples into `graph`; reject quads naming another graph |
| `GraphTarget::Dataset` | Preserve N-Quads/TriG graph names |

`GraphTarget::Dataset` is valid only for N-Quads and TriG. Choosing it for a
graph-only syntax returns `Error::Unsupported`.

```rust,no_run
use oxiland::io::{GraphTarget, Parser, Syntax};

let parser = Parser::for_syntax(Syntax::NQuads)
    .graph_target(GraphTarget::Dataset);
let stream = parser.parse_path("snapshot.nq")?;
# Ok::<(), oxiland::Error>(())
```

## Load into a model

Three APIs make failure and memory behavior visible:

| API | Buffering | Failure behavior | Use when |
|---|---|---|---|
| `load_into` | Streaming | Successful inserts before an error remain | Partial progress is intentional |
| `load_collecting` | Complete input | Parse failure leaves model unchanged; insert rollback is best effort | Input is bounded and parse-first semantics are enough |
| `load_transactional` | Complete input | Parse then commit as one model transaction | Atomic import is required |

Each has a corresponding `load_path_*` method. Return values count processed
input quads, including duplicates already present in the RDF set.

```rust,no_run
use oxiland::io::{GraphTarget, Parser, Syntax};
use oxiland::Model;

fn import_snapshot(model: &Model) -> oxiland::Result<usize> {
    Parser::for_syntax(Syntax::NQuads)
        .graph_target(GraphTarget::Dataset)
        .load_path_transactional(model, "snapshot.nq")
}
```

On persistent models, progressive inserts are durable as they succeed. A later
parse failure does not undo them. Prefer transactional load for replacement,
deployment, and externally supplied imports.

## Serialize RDF

```rust
use oxiland::io::{Serializer, Syntax};
use oxiland::Model;

# fn main() -> oxiland::Result<()> {
let model = Model::new()?;
let text = Serializer::for_syntax(Syntax::Turtle)
    .with_prefix("ex", "https://example.com/")?
    .serialize_model_to_string(&model)?;
assert!(text.is_empty());
# Ok(())
# }
```

`serialize_model_to_writer` streams the model without buffering a second full
dataset. `serialize_model_to_path` uses buffered file output.
`serialize_quads_to_writer` and `serialize_triples_to_writer` accept owned
iterators.

Turtle, TriG, and RDF/XML support configured prefixes and base IRIs. N-Triples
and N-Quads reject those settings. Graph-only syntaxes reject a model containing
named-graph statements; use N-Quads or TriG to preserve the dataset.

For large output, write to a file or network writer instead of asking for a
`String`:

```rust,no_run
use std::fs::File;
use std::io::BufWriter;
use oxiland::io::{Serializer, Syntax};
# use oxiland::Model;
# let model = Model::new()?;

let file = File::create("snapshot.nq")?;
Serializer::for_syntax(Syntax::NQuads)
    .serialize_model_to_writer(BufWriter::new(file), &model)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Error categories

| Error | Typical cause |
|---|---|
| `Error::InvalidRdf` | Invalid base IRI or prefix IRI |
| `Error::Parse` | Malformed RDF, sometimes with `SourceLocation` |
| `Error::Serialize` | Invalid serialization configuration or value |
| `Error::Io` | Reader, writer, or filesystem failure |
| `Error::Unsupported` | Unknown syntax, incompatible graph target, or invalid format capability |

Do not assume parse errors happen at iterator construction. Handle errors in
the loop and define whether already processed work is retained.

## Production guidance

- Set explicit input-size and processing limits for untrusted documents.
- Prefer dataset formats for backups that must preserve named graphs.
- Write critical exports to a temporary application-controlled path and promote
  them only after successful serialization and filesystem handling.
- Avoid logging complete RDF payloads or literal values by default.
- Test import and export with representative data, syntax, graph structure, and
  malformed-input cases.

See [Streams and iterators](streams.md), [Persistence](persistence.md), and
[Rust production operations](rust-production.md).
