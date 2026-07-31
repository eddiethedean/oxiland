"""SPARQL Update INSERT/DELETE over an in-memory model."""

from oxiland import Model, query, update

model = Model()
update(
    model,
    'INSERT DATA { <https://example.com/alice> <https://example.com/name> "Alice" }',
)
assert query(
    model,
    'ASK { <https://example.com/alice> <https://example.com/name> "Alice" }',
) is True
update(
    model,
    'DELETE DATA { <https://example.com/alice> <https://example.com/name> "Alice" }',
)
assert model.is_empty()
print("update ok")
