"""Persistent Fjall model with a transaction."""

import tempfile
from pathlib import Path

from oxiland import Literal, Model, NamedNode, Triple

with tempfile.TemporaryDirectory() as tmp:
    path = Path(tmp) / "store"
    model = Model.open(path)
    statement = Triple(
        NamedNode("https://example.com/alice"),
        NamedNode("https://example.com/name"),
        Literal("Alice"),
    )
    with model.transaction() as txn:
        txn.add(statement)
    model.sync()
    reopened = Model.open(path, create=False)
    assert reopened.contains(statement)
print("persistent ok")
