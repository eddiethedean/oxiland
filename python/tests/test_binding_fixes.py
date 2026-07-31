"""Coverage for binding correctness fixes (0.7.0)."""

from __future__ import annotations

from pathlib import Path

import pytest

from oxiland import (
    DefaultGraph,
    Literal,
    Model,
    NamedNode,
    OpenStoreError,
    ParseError,
    Quad,
    Triple,
    UnsupportedError,
    parse,
    query,
)


class _PathLike:
    def __init__(self, path: Path) -> None:
        self._path = path

    def __fspath__(self) -> str:
        return str(self._path)


def test_pathlike_open(tmp_path: Path) -> None:
    path = tmp_path / "store"
    model = Model.open(_PathLike(path))
    statement = Triple(
        NamedNode("https://example.com/s"),
        NamedNode("https://example.com/p"),
        Literal("v"),
    )
    model.add(statement)
    model.sync()
    reopened = Model.open(_PathLike(path), create=False)
    assert reopened.contains(statement) is True


def test_triple_quad_equality_and_hash() -> None:
    s = NamedNode("https://example.com/s")
    p = NamedNode("https://example.com/p")
    o = Literal("v")
    t1 = Triple(s, p, o)
    t2 = Triple(s, p, o)
    t3 = Triple(s, p, Literal("other"))
    assert t1 == t2
    assert t1 != t3
    assert hash(t1) == hash(t2)
    assert {t1, t2} == {t1}

    q1 = Quad(s, p, o)
    q2 = Quad(s, p, o, DefaultGraph())
    q3 = Quad(s, p, o, NamedNode("https://example.com/g"))
    assert q1 == q2
    assert q1 != q3
    assert hash(q1) == hash(q2)


def test_solution_unbound_returns_none() -> None:
    model = Model()
    model.add(
        Triple(
            NamedNode("https://example.com/alice"),
            NamedNode("https://example.com/name"),
            Literal("Alice"),
        )
    )
    rows = list(
        query(
            model,
            """
            SELECT ?name ?missing WHERE {
              <https://example.com/alice> <https://example.com/name> ?name
            }
            """,
        )
    )
    assert len(rows) == 1
    row = rows[0]
    assert row["name"].value == "Alice"
    assert row["missing"] is None
    assert row.get("missing") is None
    with pytest.raises(KeyError):
        _ = row["not_a_var"]


def test_default_graph_accepts_tuple_and_list() -> None:
    model = Model()
    g = NamedNode("https://example.com/g")
    model.add(
        Triple(
            NamedNode("https://example.com/s"),
            NamedNode("https://example.com/p"),
            Literal("in-g"),
        ),
        graph=g,
    )
    # Named graph only; default graph empty — union / named default_graph needed.
    rows_list = list(
        query(
            model,
            "SELECT ?o WHERE { ?s ?p ?o }",
            default_graph=[g],
        )
    )
    rows_tuple = list(
        query(
            model,
            "SELECT ?o WHERE { ?s ?p ?o }",
            default_graph=(g,),
        )
    )
    assert len(rows_list) == 1
    assert rows_list[0]["o"].value == "in-g"
    assert len(rows_tuple) == 1


def test_default_graph_rejects_str() -> None:
    model = Model()
    with pytest.raises(TypeError):
        query(model, "ASK { ?s ?p ?o }", default_graph="https://example.com/g")


def test_vocab_importable() -> None:
    import importlib

    import oxiland

    rdf = importlib.import_module("oxiland.vocab.rdf")
    rdfs = importlib.import_module("oxiland.vocab.rdfs")
    xsd = importlib.import_module("oxiland.vocab.xsd")
    assert rdf.type.endswith("type")
    assert rdfs.label.endswith("label")
    assert xsd.string.endswith("string")
    assert oxiland.vocab.rdf.type == rdf.type


def test_parse_error_attrs() -> None:
    with pytest.raises(ParseError) as excinfo:
        list(parse("@@@", "turtle"))
    err = excinfo.value
    assert isinstance(err.message, str)
    assert err.message
    assert hasattr(err, "location")


def test_open_store_error_attrs(tmp_path: Path) -> None:
    missing = tmp_path / "no-such-store"
    with pytest.raises(OpenStoreError) as excinfo:
        Model.open(missing, create=False)
    err = excinfo.value
    assert isinstance(err.path, str)
    assert "no-such-store" in err.path
    assert isinstance(err.message, str)


def test_transaction_requires_enter() -> None:
    model = Model()
    txn = model.transaction()
    with pytest.raises(UnsupportedError):
        txn.add(
            Triple(
                NamedNode("https://example.com/s"),
                NamedNode("https://example.com/p"),
                Literal("v"),
            )
        )


def test_transaction_remove_quad() -> None:
    model = Model()
    quad = Quad(
        NamedNode("https://example.com/s"),
        NamedNode("https://example.com/p"),
        Literal("v"),
    )
    model.insert_quad(quad)
    with model.transaction() as txn:
        txn.remove_quad(quad)
    assert model.is_empty() is True


def test_load_read_path_like(tmp_path: Path) -> None:
    path = tmp_path / "data.nt"
    path.write_text(
        '<https://example.com/s> <https://example.com/p> "v" .\n',
        encoding="utf-8",
    )
    model = Model()
    from oxiland import load_path

    assert load_path(model, _PathLike(path), "ntriples") == 1
