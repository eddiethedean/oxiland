"""Parse Turtle and serialize N-Triples."""

from oxiland import Model, load, serialize

model = Model()
n = load(
    model,
    '<https://example.com/alice> <https://example.com/name> "Alice" .',
    "turtle",
)
assert n == 1
text = serialize(model, "ntriples")
assert "Alice" in text
print("parse_serialize ok")
