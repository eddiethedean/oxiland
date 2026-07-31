# Python models and RDF terms

Oxiland represents RDF values with immutable Python objects and stores triples
or quads in a `Model`. Constructors validate RDF constraints at the boundary so
invalid values fail before they enter a dataset.

## Terms

```python
from oxiland import BlankNode, Literal, NamedNode

person = NamedNode("https://example.com/person/42")
anonymous = BlankNode()              # generated identifier
stable_blank = BlankNode("address") # caller-provided identifier
plain = Literal("Alice")
localized = Literal("Bonjour", language="fr")
typed = Literal(
    "42",
    datatype=NamedNode("http://www.w3.org/2001/XMLSchema#integer"),
)
```

| Type | Accepted position | Important properties |
|---|---|---|
| `NamedNode` | subject, predicate, object, graph | `.value` is the validated IRI |
| `BlankNode` | subject, object, graph | `.value` is the blank-node identifier |
| `Literal` | object | `.value`, `.language`, `.datatype` |
| `DefaultGraph` | graph | explicit default-graph marker |

`Literal` accepts either `language=` or `datatype=`, never both. Terms, triples,
and quads support equality and hashing, so they can be dictionary keys and set
members.

```python
from oxiland import InvalidRdfError

try:
    NamedNode("not an absolute IRI")
except InvalidRdfError:
    pass
```

## Triples, quads, and graphs

```python
from oxiland import DefaultGraph, Quad, Triple

name = NamedNode("https://schema.org/name")
statement = Triple(person, name, Literal("Alice"))

default_quad = Quad(person, name, Literal("Alice"), DefaultGraph())
named_quad = Quad(
    person,
    name,
    Literal("Alice"),
    NamedNode("https://example.com/graph/people"),
)
```

Passing no graph or passing `None` selects the default graph. `Quad.graph`
always returns a graph object; default-graph quads return `DefaultGraph()`.

## Create and mutate a model

```python
from oxiland import Model

model = Model()

inserted = model.add(statement)
assert inserted is True
assert model.add(statement) is False  # RDF datasets are sets
assert model.contains(statement)
assert len(model) == 1

removed = model.remove(statement)
assert removed is True
assert model.is_empty()
```

`add()`, `insert_quad()`, `remove()`, and `remove_quad()` return whether the
dataset changed. Use `clear_graph(graph)` for one graph and `clear()` for the
entire dataset.

Named-graph operations accept a `NamedNode` or `BlankNode`:

```python
graph = NamedNode("https://example.com/graph/people")
model.add(statement, graph=graph)
assert model.contains(statement, graph=graph)
```

## Pattern matching

`Model.find()` returns a lazy iterator of `Quad` objects. Omitted fields are
wildcards; supplied fields must match exactly.

```python
matches = model.find(
    subject=person,
    predicate=name,
    graph=graph,
)

for quad in matches:
    print(quad.object.value)
```

The keyword fields are `subject`, `predicate`, `object`, and `graph`. Use
`DefaultGraph()` to restrict a search to the default graph; leaving `graph`
unset searches across graphs.

## Transactions

Use a transaction when several mutations form one logical write:

```python
with model.transaction() as tx:
    tx.remove(statement, graph=graph)
    tx.add(statement)
```

Available transaction mutations are `add`, `insert_quad`, `remove`,
`remove_quad`, `clear`, and `clear_graph`.

The transaction contract is:

- mutations are buffered until the context exits normally;
- an exception rolls back the complete buffered operation set;
- commit errors propagate as typed Oxiland exceptions;
- methods used before `__enter__` fail with `UnsupportedError`;
- nested transaction contexts on the same model fail with `UnsupportedError`.

Do not retain a transaction object after its `with` block. Start a new context
for the next logical write.

## Persistent models

```python
from pathlib import Path

path = Path("var/data/knowledge-graph")
model = Model.open(path)
print(model.backend)  # "fjall"

# Optional: select the storage backend (default is "fjall")
# model = Model.open(path, backend="fjall")

with model.transaction() as tx:
    tx.add(statement)

model.sync()
```

`Model.open(path, read_only=False, create=True, backend="fjall")` controls
whether writes are allowed, whether a missing store may be created, and which
compiled backend opens the path. The default `backend=` is `"fjall"`. Use
`create=False` in deployments that must fail instead of silently initializing
an empty dataset.

See [Production operations](python-production.md) for ownership, backup,
restore, and upgrade guidance.
