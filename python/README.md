# Oxiland (Python)

Pythonic RDF models, SPARQL, and stream-oriented I/O over the Oxiland safe Rust
facade (Oxigraph-backed). This is **not** a drop-in for legacy Redland Python
bindings and does not claim rdflib identity (ADR-017).

## Install

```console
pip install oxiland
```

From a checkout (development):

```console
cd python
pip install maturin pytest
maturin develop
pytest
```

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

See [Python guide](https://oxiland.readthedocs.io/en/latest/users/python/).

## License

Apache-2.0 OR MIT
