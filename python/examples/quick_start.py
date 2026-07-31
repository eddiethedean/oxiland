"""Quick start: build a model and run ASK."""

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
print("quick_start ok")
