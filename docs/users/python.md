# Oxiland for Python

Oxiland is a complete Python toolkit for local RDF datasets: create and
validate terms, manage default and named graphs, query with SPARQL, stream RDF
files, and keep durable stores on disk.

The Python package is distributed independently on PyPI, has no required
Python dependencies, and includes type information for static analysis and IDE
completion.

## Start in five minutes

Install a wheel into a virtual environment:

```console
python -m venv .venv
source .venv/bin/activate
python -m pip install oxiland
```

On Windows PowerShell, activate with `.venv\Scripts\Activate.ps1`.

Create a dataset and query it:

```python
from oxiland import Literal, Model, NamedNode, Triple, query

EX = "https://example.com/"
model = Model()

model.add(
    Triple(
        NamedNode(f"{EX}alice"),
        NamedNode(f"{EX}name"),
        Literal("Alice", language="en"),
    )
)

assert query(model, "ASK { ?s ?p ?o }") is True

for row in query(model, "SELECT ?s ?name WHERE { ?s <https://example.com/name> ?name }"):
    print(row["s"].value, row["name"].value)
```

## Package capabilities

| Area | Production-facing capability |
|---|---|
| RDF values | Validated IRIs, blank nodes, literals, triples, quads, and graph names |
| Models | In-memory and persistent datasets with default and named graphs |
| Writes | Idempotent add/remove, clear operations, and atomic transactions |
| Reads | Exact containment, lazy pattern matching, and dataset length |
| RDF I/O | Turtle, N-Triples, N-Quads, TriG, and RDF/XML |
| SPARQL | ASK, SELECT, CONSTRUCT, DESCRIBE, Update, and result serialization |
| Operations | Read-only open, explicit sync, N-Quads backup/restore, typed failures |
| Developer experience | CPython 3.10–3.14 wheels and bundled PEP 561 type information |

## Choose the right model

```python
from pathlib import Path
from oxiland import Model

scratch = Model()                              # process-local, in memory
catalog = Model.open(Path("var/catalog"))     # durable local dataset
replica = Model.open(
    Path("var/catalog"),
    read_only=True,
    create=False,
)
```

Use an in-memory model for request-scoped transformations, tests, and caches.
Use a persistent model when data must survive restarts. A persistent store is a
local embedded database, not a remote service: your application owns its path,
permissions, backup policy, and process lifecycle.

## Atomic writes

```python
from oxiland import Literal, Model, NamedNode, Triple

EX = "https://example.com/"
catalog = Model()

with catalog.transaction() as tx:
    tx.clear_graph(NamedNode(f"{EX}staging"))
    tx.add(
        Triple(
            NamedNode(f"{EX}alice"),
            NamedNode(f"{EX}status"),
            Literal("active"),
        ),
        graph=NamedNode(f"{EX}staging"),
    )
```

The block commits as one unit. If Python leaves it with an exception, no
buffered operation is committed. Transactions must be used as context managers
and cannot be nested on the same `Model`.

## Stream large results

`Model.find()`, `parse()`, `parse_path()`, SELECT, CONSTRUCT, and DESCRIBE are
lazy. Process their iterators directly instead of converting them to lists when
the result might be large:

```python
from oxiland import NamedNode

EX = "https://example.com/"
for quad in catalog.find(predicate=NamedNode(f"{EX}status")):
    print(quad)
```

Dropping an iterator early is supported. The iterator owns the state required
to continue reading.

## Python documentation track

1. [Installation and compatibility](python-installation.md)
2. [Models and RDF terms](python-models.md)
3. [RDF I/O and SPARQL](python-data.md)
4. [Production operations](python-production.md)
5. [API reference](python-api.md)
6. [Runnable examples](examples.md#python-pythonexamples)

The [support policy](../support.md), [security policy](../security.md),
[known limitations](limitations.md), [upgrading](upgrading.md), and
[changelog](https://github.com/eddiethedean/oxiland/blob/main/CHANGELOG.md)
apply to the Python distribution.

## Scope

The public Python contract is the API documented in this track and shipped in
the wheel's PEP 561 stubs. The implementation uses a native RDF engine, but
Python callers do not need a Rust toolchain or knowledge of the Rust API.

Oxiland is not an rdflib adapter or a drop-in for historical Redland Python
bindings. Query cancellation tokens, custom storage engines, and rdflib object
conversion are not exposed. Unsupported operations fail explicitly instead of
silently changing semantics.
