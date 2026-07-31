"""Core model CRUD and streaming find tests."""

from __future__ import annotations

import pytest

from oxiland import (
    InvalidRdfError,
    Literal,
    Model,
    NamedNode,
    Triple,
)


def _alice() -> Triple:
    return Triple(
        NamedNode("https://example.com/alice"),
        NamedNode("https://example.com/name"),
        Literal("Alice"),
    )


def test_model_add_contains_len() -> None:
    model = Model()
    statement = _alice()
    assert model.add(statement) is True
    assert model.add(statement) is False
    assert model.contains(statement) is True
    assert len(model) == 1
    assert model.is_empty() is False


def test_named_graph_and_find() -> None:
    model = Model()
    graph = NamedNode("https://example.com/people")
    statement = _alice()
    assert model.add(statement, graph=graph) is True
    matches = list(model.find(subject=NamedNode("https://example.com/alice")))
    assert len(matches) == 1
    graph_term = matches[0].graph
    assert isinstance(graph_term, NamedNode)
    assert graph_term.value == graph.value


def test_find_early_termination() -> None:
    model = Model()
    for i in range(20):
        model.add(
            Triple(
                NamedNode(f"https://example.com/s{i}"),
                NamedNode("https://example.com/p"),
                Literal(str(i)),
            )
        )
    it = model.find()
    first = next(it)
    assert first is not None
    del it  # drop without exhausting


def test_invalid_named_node() -> None:
    with pytest.raises(InvalidRdfError):
        NamedNode("not an iri")


def test_remove_and_clear() -> None:
    model = Model()
    statement = _alice()
    model.add(statement)
    assert model.remove(statement) is True
    assert model.is_empty() is True
    model.add(statement)
    model.clear()
    assert model.is_empty() is True


def test_transaction_commit_and_rollback() -> None:
    model = Model()
    statement = _alice()
    with model.transaction() as txn:
        txn.add(statement)
    assert model.contains(statement) is True

    model.clear()
    try:
        with model.transaction() as txn:
            txn.add(statement)
            raise RuntimeError("boom")
    except RuntimeError:
        pass
    assert model.is_empty() is True
