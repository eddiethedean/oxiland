"""SPARQL query / update tests."""

from __future__ import annotations

import pytest

from oxiland import (
    Literal,
    Model,
    NamedNode,
    SparqlParseError,
    Triple,
    query,
    serialize_results,
    update,
)


def _seed() -> Model:
    model = Model()
    model.add(
        Triple(
            NamedNode("https://example.com/alice"),
            NamedNode("https://example.com/name"),
            Literal("Alice"),
        )
    )
    return model


def test_ask_select_construct() -> None:
    model = _seed()
    assert query(model, "ASK { ?s ?p ?o }") is True
    rows = list(query(model, "SELECT ?name WHERE { ?s <https://example.com/name> ?name }"))
    assert rows[0]["name"].value == "Alice"
    triples = list(
        query(model, "CONSTRUCT { ?s <https://example.com/label> ?name } WHERE { ?s <https://example.com/name> ?name }")
    )
    assert len(triples) == 1


def test_select_early_stop() -> None:
    model = Model()
    for i in range(15):
        model.add(
            Triple(
                NamedNode(f"https://example.com/s{i}"),
                NamedNode("https://example.com/p"),
                Literal(str(i)),
            )
        )
    it = query(model, "SELECT ?s WHERE { ?s ?p ?o }")
    assert next(it) is not None
    del it


def test_construct_early_stop() -> None:
    model = _seed()
    it = query(
        model,
        "CONSTRUCT { ?s <https://example.com/label> ?name } "
        "WHERE { ?s <https://example.com/name> ?name }",
    )
    assert next(it) is not None
    del it


def test_update_insert() -> None:
    model = Model()
    update(
        model,
        'INSERT DATA { <https://example.com/bob> <https://example.com/name> "Bob" }',
    )
    assert len(model) == 1


def test_serialize_results_json() -> None:
    model = _seed()
    text = serialize_results(
        model,
        "SELECT ?name WHERE { ?s <https://example.com/name> ?name }",
        "json",
    )
    assert "Alice" in text


def test_sparql_parse_error() -> None:
    model = Model()
    with pytest.raises(SparqlParseError):
        query(model, "NOT SPARQL")
