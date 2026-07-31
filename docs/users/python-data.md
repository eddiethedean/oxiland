# Python RDF I/O and SPARQL

Oxiland exposes Python functions for parsing, loading, serializing, querying,
and updating RDF. Inputs accept Python strings, bytes, and `os.PathLike` paths;
large read paths return lazy iterators.

## Supported RDF syntaxes

| Syntax | Name | Extension | Can represent named graphs |
|---|---|---|---|
| Turtle | `turtle` | `.ttl` | No |
| N-Triples | `ntriples` | `.nt` | No |
| N-Quads | `nquads` | `.nq` | Yes |
| TriG | `trig` | `.trig` | Yes |
| RDF/XML | `rdfxml` | `.rdf` | No |

Pass a name string or a `Syntax` constant. `Syntax.from_name()`,
`Syntax.from_media_type()`, and `Syntax.from_extension()` validate external
metadata without starting a parse.

JSON-LD and N3 are not supported in 0.7.0. Unknown or ambiguous formats raise
`UnsupportedError`; Oxiland does not guess from document contents.

## Stream a document

```python
from oxiland import parse, parse_path

for quad in parse(
    '<https://example.com/s> <https://example.com/p> "value" .',
    "turtle",
):
    print(quad)

for quad in parse_path("document.ttl"):
    process(quad)
```

`parse_path()` infers syntax from the extension when `syntax` is omitted.
Parsing is lazy: syntax errors may be raised while advancing the iterator, not
when the iterator is constructed.

Use `base_iri=` for relative IRIs. Use `graph=` to place graph-format input in
a named graph.

!!! note "Named-graph input in 0.7.0"

    N-Quads and TriG are supported formats, and serialization preserves named
    graphs. The Python parser does not yet expose the dataset graph target for
    arbitrary named-graph input. Without `graph=`, named-graph records raise
    `ParseError`; with `graph=`, input must be compatible with that target.
    Programmatic named-graph CRUD and N-Quads/TriG output are fully available.

## Load into a model

```python
from oxiland import Model, load, load_path

model = Model()

count = load(model, turtle_text, "turtle")
count += load_path(model, "catalog.trig", transactional=True)
```

Choose failure behavior explicitly:

| Mode | Call | Failure behavior | Best for |
|---|---|---|---|
| Collecting | default (`collecting=True`) | Parse first, then insert; parse failure leaves the model unchanged | Normal bounded imports |
| Transactional | `transactional=True` | Import is committed atomically | Persistent or replace-sensitive workflows |
| Progressive | `collecting=False` | Successful statements before an error may remain | Very large trusted inputs where partial progress is acceptable |

`transactional=True` takes precedence over `collecting`. The return value is
the number of statements processed. Loading merges RDF into the existing
dataset; call `clear()` in the same controlled workflow when replacement is
intended.

## Serialize a dataset

```python
from oxiland import serialize, serialize_path

turtle = serialize(
    model,
    "turtle",
    prefixes={"ex": "https://example.com/"},
)
serialize_path(model, "snapshot.nq")
```

`serialize()` returns `str`. `serialize_path()` writes directly to a path and
infers syntax from the extension when omitted. Prefer N-Quads or TriG when the
dataset includes named graphs.

## Query result types

```python
from oxiland import query

answer = query(model, "ASK { ?s ?p ?o }")

rows = query(
    model,
    "SELECT ?s ?label WHERE { ?s <https://schema.org/name> ?label }",
    limit=500,
    offset=0,
)
for row in rows:
    subject = row["s"]
    label = row.get("label")

triples = query(
    model,
    "CONSTRUCT { ?s <https://example.com/seen> ?o } WHERE { ?s ?p ?o }",
)
for triple in triples:
    process(triple)
```

| Query form | Python result |
|---|---|
| ASK | `bool` |
| SELECT | lazy `SolutionsIter` of `Solution` |
| CONSTRUCT / DESCRIBE | lazy `TriplesIter` of `Triple` |

A `Solution` supports access by variable name or position. A selected but
unbound variable returns `None`. An unknown variable name or invalid position
raises `KeyError`. `variables()` returns the result variable order.

Use `base_iri=`, `limit=`, and `offset=` for query configuration. Use
`default_graph=` with one graph name or a sequence of graph names; a Python
string is intentionally rejected because it is not an RDF graph object.
`default_graph_as_union=True` treats all graphs as the default query graph.

## SPARQL Update

```python
from oxiland import update

update(
    model,
    '''
    DELETE { ?s <https://example.com/status> "pending" }
    INSERT { ?s <https://example.com/status> "active" }
    WHERE  { ?s <https://example.com/status> "pending" }
    ''',
)
```

Update accepts `base_iri=`, `default_graph=`, and
`default_graph_as_union=`. A parse failure raises `SparqlParseError`; an
execution failure raises `SparqlEvaluationError` or a storage exception.

## Serialize ASK and SELECT results

```python
from oxiland import serialize_results

payload = serialize_results(
    model,
    "SELECT ?s WHERE { ?s ?p ?o }",
    "json",
    limit=100,
)
```

SPARQL result formats are JSON, XML, CSV, and TSV. `serialize_results()` is for
ASK and SELECT. Serialize CONSTRUCT and DESCRIBE graph results as RDF instead.

## Resource control

Use query-level filters and `limit=` for caller-facing SELECT or graph queries,
and consume lazy iterators incrementally. The 0.7.0 Python API does not expose a
query cancellation token or wall-clock timeout. Do not execute unrestricted
SPARQL supplied by untrusted clients in a latency-sensitive worker; enforce
limits and isolation at the application boundary.
