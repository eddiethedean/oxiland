# Python API reference

This page is the compact reference for Oxiland 0.7.0. All public symbols are
available from `oxiland` unless a vocabulary submodule is shown. The wheel
ships PEP 561 declarations, so the signatures below are also available through
IDE completion and static type checkers.

## Import and version

```python
import oxiland
from oxiland import Model, query

print(oxiland.__version__)
```

`__version__: str` is the installed distribution version.

## Type aliases

| Alias | Accepted types |
|---|---|
| `PathArg` | `str | os.PathLike[str]` |
| `Term` | `NamedNode | BlankNode | Literal` |
| `Subject` | `NamedNode | BlankNode` |
| `GraphName` | `DefaultGraph | NamedNode | BlankNode | None` |

## RDF terms

### `NamedNode`

```python
NamedNode(iri: str)
named_node(iri: str) -> NamedNode
```

Creates a validated absolute IRI. `.value: str` returns its lexical IRI.

### `BlankNode`

```python
BlankNode(id: str | None = None)
blank_node(id: str | None = None) -> BlankNode
```

Creates a generated blank node when `id` is omitted or validates a supplied
identifier. `.value: str` returns the identifier.

### `Literal`

```python
Literal(
    value: str,
    *,
    language: str | None = None,
    datatype: NamedNode | None = None,
)
```

Properties are `.value: str`, `.language: str | None`, and
`.datatype: NamedNode`. Supplying both `language` and `datatype` raises
`ValueError`.

### `Triple` and `Quad`

```python
Triple(subject: Subject, predicate: NamedNode, object: Term)
Quad(subject: Subject, predicate: NamedNode, object: Term, graph: GraphName = None)
DefaultGraph()
```

`Triple` exposes `.subject`, `.predicate`, and `.object`. `Quad` additionally
exposes `.graph`, returning a concrete `DefaultGraph`, `NamedNode`, or
`BlankNode`. These value objects are immutable, comparable, and hashable.

## `Model`

```python
Model()
Model.open(path: PathArg, *, read_only: bool = False, create: bool = True) -> Model
Model.migrate_legacy_store(path: PathArg) -> Model
```

`Model()` creates an in-memory dataset. `Model.open()` opens a persistent local
dataset. `.backend` is `"memory"` or `"fjall"`.

### Dataset operations

| Method | Return | Behavior |
|---|---|---|
| `add(statement, graph=None)` | `bool` | Insert a triple; return whether the dataset changed |
| `insert_quad(quad)` | `bool` | Insert an explicit quad |
| `remove(statement, graph=None)` | `bool` | Remove a triple; return whether the dataset changed |
| `remove_quad(quad)` | `bool` | Remove an explicit quad |
| `contains(statement, graph=None)` | `bool` | Test exact membership |
| `clear()` | `None` | Remove all quads |
| `clear_graph(graph)` | `None` | Remove all quads from one graph |
| `len(model)` | `int` | Return statement count |
| `is_empty()` | `bool` | Test whether the dataset has no statements |
| `sync()` | `None` | Synchronize persistent state |
| `export_nquads(path)` | `None` | Write a portable dataset backup |
| `import_nquads(path)` | `int` | Merge an N-Quads file and return processed count |

### Pattern matching

```python
Model.find(
    *,
    subject: Subject | None = None,
    predicate: NamedNode | None = None,
    object: Term | None = None,
    graph: GraphName = None,
) -> FindIter
```

Returns a lazy iterator of `Quad`. Omitted term fields are wildcards. An
omitted graph searches the dataset; pass `DefaultGraph()` to select only the
default graph.

### Transactions

```python
Model.transaction() -> Transaction
```

`Transaction` is a context manager with `add`, `insert_quad`, `remove`,
`remove_quad`, `clear`, and `clear_graph`. Normal exit commits; exceptional
exit rolls back. It must be entered before mutation and cannot be nested on the
same model.

## RDF syntax and I/O

### `Syntax`

Constants: `Syntax.TURTLE`, `Syntax.NTRIPLES`, `Syntax.NQUADS`, `Syntax.TRIG`,
and `Syntax.RDFXML`.

```python
Syntax.from_name(name: str) -> Syntax
Syntax.from_media_type(media_type: str) -> Syntax
Syntax.from_extension(extension: str) -> Syntax
```

Properties are `.name`, `.media_type`, and `.extension`.

### Parse

```python
parse(
    data: str | bytes,
    syntax: Syntax | str,
    *,
    base_iri: str | None = None,
    graph: GraphName = None,
) -> ParseIter

parse_path(
    path: PathArg,
    syntax: Syntax | str | None = None,
    *,
    base_iri: str | None = None,
    graph: GraphName = None,
) -> ParseIter
```

Both return lazy iterators of `Quad`. Path syntax is inferred from the extension
when omitted.

### Load

```python
load(
    model: Model,
    data: str | bytes,
    syntax: Syntax | str,
    *,
    collecting: bool = True,
    transactional: bool = False,
    base_iri: str | None = None,
    graph: GraphName = None,
) -> int

load_path(
    model: Model,
    path: PathArg,
    syntax: Syntax | str | None = None,
    *,
    collecting: bool = True,
    transactional: bool = False,
    base_iri: str | None = None,
    graph: GraphName = None,
) -> int
```

The return value is the processed statement count. `transactional=True`
selects atomic import; otherwise `collecting=True` parses before insertion and
`collecting=False` inserts progressively.

### Serialize

```python
serialize(
    model: Model,
    syntax: Syntax | str,
    *,
    base_iri: str | None = None,
    prefixes: Mapping[str, str] | None = None,
) -> str

serialize_path(
    model: Model,
    path: PathArg,
    syntax: Syntax | str | None = None,
    *,
    base_iri: str | None = None,
) -> None
```

## SPARQL

```python
query(
    model: Model,
    sparql: str,
    *,
    base_iri: str | None = None,
    limit: int | None = None,
    offset: int | None = None,
    default_graph: object = None,
    default_graph_as_union: bool = False,
) -> bool | SolutionsIter | TriplesIter
```

ASK returns `bool`; SELECT returns `SolutionsIter`; CONSTRUCT and DESCRIBE
return `TriplesIter`.

`Solution` supports `row[name]`, `row[position]`, `row.get(name)`,
`row.variables()`, and `len(row)`. Selected but unbound values are `None`.

```python
update(
    model: Model,
    sparql: str,
    *,
    base_iri: str | None = None,
    default_graph: object = None,
    default_graph_as_union: bool = False,
) -> None

serialize_results(
    model: Model,
    sparql: str,
    format: str,
    *,
    base_iri: str | None = None,
    limit: int | None = None,
    offset: int | None = None,
) -> str
```

Result serialization supports JSON, XML, CSV, and TSV for ASK and SELECT.

## Utilities

### Digests

```python
DigestAlgorithm.MD5
DigestAlgorithm.SHA1
DigestAlgorithm.SHA256
DigestAlgorithm.from_name(name: str) -> DigestAlgorithm
digest_hex(algorithm: DigestAlgorithm | str, data: str | bytes) -> str
digest_bytes(algorithm: DigestAlgorithm | str, data: str | bytes) -> bytes
```

### Namespaces and vocabularies

```python
Namespace(prefix: str, base: str)
Namespace.expand(local: str) -> NamedNode
```

`Namespace` exposes `.prefix` and `.base`. Bundled vocabulary modules include
`rdf`, `rdfs`, `xsd`, `owl`, and `dc`:

```python
from oxiland import vocab
from oxiland.vocab import rdf, rdfs, xsd

predicate = NamedNode(rdfs.label)
datatype = NamedNode(xsd.string)
assert vocab.rdf.type == rdf.type
```

## Exceptions

```text
OxilandError
├── InvalidRdfError
├── ParseError              # .message, .location
├── SerializeError
├── SparqlParseError
├── SparqlEvaluationError
├── StorageError
├── OpenStoreError          # .path, .message
├── IoError
└── UnsupportedError
```

Catch specific subclasses for recoverable conditions or `OxilandError` at an
application boundary.

## Authoritative declarations

- [Bundled stub source](https://github.com/eddiethedean/oxiland/blob/main/python/oxiland.pyi)
- [Models and RDF terms](python-models.md)
- [RDF I/O and SPARQL](python-data.md)
- [Production operations](python-production.md)

The stub shipped inside the installed wheel is authoritative for the exact
installed version.
