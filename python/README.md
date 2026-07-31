# Oxiland for Python

[![CI](https://github.com/eddiethedean/oxiland/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/oxiland/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/oxiland.svg)](https://pypi.org/project/oxiland/)
[![Python versions](https://img.shields.io/pypi/pyversions/oxiland.svg)](https://pypi.org/project/oxiland/)
[![Guides](https://img.shields.io/readthedocs/oxiland?label=Guides)](https://oxiland.readthedocs.io/en/latest/users/python/)
[![License](https://img.shields.io/pypi/l/oxiland.svg)](https://github.com/eddiethedean/oxiland/blob/main/LICENSE-APACHE)

Oxiland is a typed Python library for working with RDF datasets. It combines
an in-memory or persistent model, named graphs, SPARQL 1.1 queries and updates,
and streaming RDF input in one dependency-free package.

```console
python -m pip install oxiland
```

```python
from oxiland import Literal, Model, NamedNode, Triple, query

EX = "https://example.com/"

model = Model()
model.add(
    Triple(
        NamedNode(f"{EX}alice"),
        NamedNode(f"{EX}name"),
        Literal("Alice"),
    )
)

for row in query(
    model,
    f"SELECT ?name WHERE {{ <{EX}alice> <{EX}name> ?name }}",
):
    print(row["name"].value)
```

## What you get

- native RDF terms with validation, value equality, and hashing;
- default-graph and named-graph CRUD;
- in-memory models and local persistent stores;
- atomic write transactions with rollback on exceptions;
- ASK, SELECT, CONSTRUCT, DESCRIBE, and SPARQL Update;
- streaming parse, match, and query-result iterators;
- Turtle, N-Triples, N-Quads, TriG, and RDF/XML;
- PEP 561 type information for Pyright, mypy, and IDEs;
- typed exceptions for RDF, parsing, SPARQL, storage, and I/O failures.

Oxiland has no required Python dependencies. Published wheels contain the
native engine and the complete Python API.

## Compatibility

| Requirement | Support |
|---|---|
| Python | CPython 3.10–3.14 |
| Operating systems | Linux, macOS, and Windows wheels |
| Typing | PEP 561 marker and bundled stubs |
| Distribution | Binary wheels; no source distribution in 0.7.0 |

Installation succeeds only when PyPI has a wheel matching the interpreter and
platform. See the [installation guide](https://oxiland.readthedocs.io/en/latest/users/python-installation/)
for deployment and source-build guidance.

## Persistent datasets and transactions

Use `Model.open()` when the dataset must survive process restarts. Paths accept
`str` and `os.PathLike` values.

```python
from pathlib import Path

from oxiland import Literal, Model, NamedNode, Triple

store = Model.open(Path("var/data/catalog"))

with store.transaction() as tx:
    tx.add(
        Triple(
            NamedNode("https://example.com/product/42"),
            NamedNode("https://schema.org/name"),
            Literal("Desk lamp"),
        )
    )

store.sync()
```

The context commits as one unit when it exits normally. An exception discards
all buffered operations. Nested transactions on the same model are rejected
with `UnsupportedError`.

## RDF input and output

```python
from oxiland import Model, load_path, serialize_path

model = Model()
loaded = load_path(model, "catalog.ttl", transactional=True)
serialize_path(model, "catalog.nq")
print(f"loaded {loaded} statements")
```

`parse()` and `parse_path()` return lazy quad iterators. `load()` and
`load_path()` provide progressive, collecting, and transactional import modes
so applications can choose memory usage and failure atomicity explicitly.
In 0.7.0, an N-Quads or TriG file containing arbitrary named graphs cannot be
imported as one dataset. Load one compatible graph at a time with `graph=`;
dataset-target parsing is not yet exposed in Python.

## SPARQL

```python
from oxiland import query, serialize_results, update

exists = query(model, "ASK { ?s ?p ?o }")

rows = query(
    model,
    "SELECT ?s ?p ?o WHERE { ?s ?p ?o } ORDER BY ?s",
    limit=100,
)
for row in rows:
    print(row["s"], row["p"], row["o"])

json_results = serialize_results(
    model,
    "SELECT ?s WHERE { ?s ?p ?o }",
    "json",
)

update(model, 'INSERT DATA { <https://example.com/s> <https://example.com/p> "v" }')
```

ASK returns `bool`; SELECT returns a lazy `SolutionsIter`; CONSTRUCT and
DESCRIBE return a lazy `TriplesIter`. Unbound selected variables have the value
`None`.

## Error handling

Catch the narrowest exception that your application can recover from:

```python
from oxiland import OpenStoreError, ParseError, StorageError, load_path

try:
    load_path(model, "incoming.ttl", transactional=True)
except ParseError as error:
    print(error.location, error.message)
except (OpenStoreError, StorageError) as error:
    raise RuntimeError("RDF store unavailable") from error
```

All package exceptions inherit from `OxilandError`. Parse failures expose
`message` and `location`; store-open failures expose `path` and `message`.

## Documentation

- [Python guide](https://oxiland.readthedocs.io/en/latest/users/python/)
- [Models and RDF terms](https://oxiland.readthedocs.io/en/latest/users/python-models/)
- [RDF I/O and SPARQL](https://oxiland.readthedocs.io/en/latest/users/python-data/)
- [Production operations](https://oxiland.readthedocs.io/en/latest/users/python-production/)
- [API reference](https://oxiland.readthedocs.io/en/latest/users/python-api/)
- [Support policy](https://oxiland.readthedocs.io/en/latest/support/)
- [Security policy](https://oxiland.readthedocs.io/en/latest/security/)

Oxiland is powered by a native implementation for predictable performance, but
its public Python API, typing contract, errors, documentation, and release
artifacts are maintained as a first-class package. It is not an rdflib
compatibility layer; integrations should use the documented Oxiland types.

## License

Apache-2.0 OR MIT
