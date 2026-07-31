"""SELECT query over an in-memory model."""

from oxiland import Literal, Model, NamedNode, Triple, query

model = Model()
model.add(
    Triple(
        NamedNode("https://example.com/alice"),
        NamedNode("https://example.com/name"),
        Literal("Alice"),
    )
)
for row in query(model, "SELECT ?name WHERE { ?s <https://example.com/name> ?name }"):
    assert row["name"].value == "Alice"
print("select ok")
