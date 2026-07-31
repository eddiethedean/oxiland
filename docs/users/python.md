# Python package

Oxiland’s PyPI package provides a **Pythonic** API over the safe Rust facade.
It is not a 1:1 mirror of Rust builders, not a Redland Python drop-in, and does
not integrate with rdflib in 0.7.

## Install

```console
pip install oxiland
```

| Requirement | Detail |
|---|---|
| CPython | **3.10–3.13** |
| Platforms | Published **wheels** for Linux / macOS / Windows (CI matrix hosts) |
| Source builds | **No sdist** on PyPI in 0.7.0 — install needs a matching wheel |

If `pip` tries to build from source and fails, you are on an unsupported
platform/Python combination for the published wheels. Use a supported CPython
or build from a git checkout with [maturin](https://www.maturin.rs/) (see
[Contributing](../contributing.md)).

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

Runnable scripts (from a checkout):

```console
cd python
python examples/quick_start.py
python examples/select.py
python examples/parse_serialize.py
python examples/persistent.py
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

Transaction methods require an active `with` block. On exception, the context
manager discards buffered operations (no commit). Format v1 reopen covers
**0.4.x–0.7.x** patch lines ([persistence](persistence.md)).

## Parse and serialize

```python
from oxiland import Model, load, serialize

model = Model()
load(model, '<https://example.com/a> <https://example.com/p> "x" .', "turtle")
print(serialize(model, "ntriples"))
```

`parse(data, syntax)` streams quads without requiring a model. Path helpers
accept `pathlib.Path` and other `PathLike` objects.

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
`TriplesIter`. Both are lazy. Unbound variables are `None` (not `KeyError`).

## Errors

Failures raise subclasses of `OxilandError` (`InvalidRdfError`, `ParseError`,
`SparqlParseError`, `StorageError`, `OpenStoreError`, `UnsupportedError`, and
others) aligned with the Rust `Error` categories. `ParseError` exposes
`.message` / `.location`; `OpenStoreError` exposes `.path` / `.message`.

## What is not mirrored

- Rust `Query` / `Parser` / `Serializer` builders → kwargs on functions
- Query cancellation tokens
- `Model.store` / advanced Oxigraph escapes
- World logging handlers
- rdflib conversion (deferred)

## See also

- [Python API landing](python-api.md)
- [Examples index](examples.md)
- Design: [0.7-python-api.md](../design/0.7-python-api.md)
