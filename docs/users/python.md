# Python package

Oxiland’s PyPI package provides a **Pythonic** API over the safe Rust facade
(ADR-017). It is not a 1:1 mirror of Rust builders, not a Redland Python
drop-in, and does not integrate with rdflib in 0.7.

Install:

```console
pip install oxiland
```

Requires CPython 3.10–3.13.

## Quick start

```python
from oxiland import Literal, Model, NamedNode, Triple, query

model = Model()
model.add(
    Triple(
        NamedNode("https://example.com/alice"),
        NamedNode("https://example.com/name"),
        Literal("Alice"),
    )
)
assert query(model, "ASK { ?s ?p ?o }") is True
```

## Models and contexts

```python
from oxiland import Model, NamedNode, Literal, Triple

model = Model()
graph = NamedNode("https://example.com/people")
model.add(
    Triple(
        NamedNode("https://example.com/alice"),
        NamedNode("https://example.com/name"),
        Literal("Alice"),
    ),
    graph=graph,
)

for quad in model.find(subject=NamedNode("https://example.com/alice")):
    print(quad.graph)
```

`Model.find(...)` returns a lazy iterator of `Quad` values.

### Persistence and transactions

```python
from pathlib import Path
from oxiland import Model, NamedNode, Literal, Triple

model = Model.open(Path("./data/store"))
with model.transaction() as txn:
    txn.add(
        Triple(
            NamedNode("https://example.com/alice"),
            NamedNode("https://example.com/name"),
            Literal("Alice"),
        )
    )
model.sync()
```

On exception, the context manager discards buffered operations (no commit).

## Parse and serialize

```python
from oxiland import Model, load, serialize

model = Model()
load(model, '<https://example.com/a> <https://example.com/p> "x" .', "turtle")
print(serialize(model, "ntriples"))
```

`parse(data, syntax)` streams quads without requiring a model. Path helpers
accept `pathlib.Path`.

## SPARQL

```python
from oxiland import query, update, serialize_results

assert query(model, "ASK { ?s ?p ?o }") is True

for row in query(model, "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"):
    print(row["s"], row["p"], row["o"])

update(model, "DELETE WHERE { ?s ?p ?o }")
print(serialize_results(model, "ASK { ?s ?p ?o }", "json"))
```

`SELECT` returns a `SolutionsIter`; `CONSTRUCT`/`DESCRIBE` return a
`TriplesIter`. Both are lazy.

## Errors

Failures raise subclasses of `OxilandError` (`InvalidRdfError`, `ParseError`,
`SparqlParseError`, `StorageError`, `UnsupportedError`, and others) aligned with
the Rust `Error` categories.

## What is not mirrored

- Rust `Query` / `Parser` / `Serializer` builders → kwargs on functions
- Query cancellation tokens
- `Model.store` / advanced Oxigraph escapes
- World logging handlers
- rdflib conversion (deferred; ADR-017)

Design detail: [0.7-python-api.md](../design/0.7-python-api.md).
