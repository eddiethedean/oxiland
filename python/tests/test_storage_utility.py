"""Persistent storage and utility tests."""

from __future__ import annotations

from pathlib import Path

from oxiland import DigestAlgorithm, Literal, Model, NamedNode, Namespace, Triple, digest_hex


def test_persistent_open_and_transaction(tmp_path: Path) -> None:
    path = tmp_path / "store"
    model = Model.open(path)
    assert model.backend == "fjall"
    statement = Triple(
        NamedNode("https://example.com/alice"),
        NamedNode("https://example.com/name"),
        Literal("Alice"),
    )
    with model.transaction() as txn:
        txn.add(statement)
    model.sync()
    reopened = Model.open(path, create=False)
    assert reopened.contains(statement) is True


def test_digest_and_namespace() -> None:
    assert digest_hex("md5", b"abc") == "900150983cd24fb0d6963f7d28e17f72"
    assert DigestAlgorithm.SHA256.name == "sha256"
    ns = Namespace("ex", "https://example.com/")
    assert ns.expand("alice").value == "https://example.com/alice"
