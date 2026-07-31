"""CONSTRUCT query over an in-memory model."""

from oxiland import Literal, Model, NamedNode, Triple, query

model = Model()
model.add(
    Triple(
        NamedNode("https://example.com/alice"),
        NamedNode("https://example.com/name"),
        Literal("Alice"),
    )
)
triples = query(
    model,
    "CONSTRUCT { ?s <https://example.com/label> ?o } "
    "WHERE { ?s <https://example.com/name> ?o }",
)
labels = list(triples)
assert len(labels) == 1
assert labels[0].predicate.value == "https://example.com/label"
print("construct ok")
