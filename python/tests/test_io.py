"""Parse / serialize / load tests."""

from __future__ import annotations

from pathlib import Path

import pytest

from oxiland import Literal, Model, NamedNode, Syntax, Triple, load, parse, serialize, serialize_path
from oxiland import UnsupportedError


def test_parse_stream_and_load() -> None:
    data = '<https://example.com/a> <https://example.com/p> "x" .'
    quads = list(parse(data, "turtle"))
    assert len(quads) == 1
    assert quads[0].object.value == "x"

    model = Model()
    assert load(model, data, Syntax.TURTLE) == 1
    assert "x" in serialize(model, "ntriples")


def test_parse_early_stop() -> None:
    data = "\n".join(
        f'<https://example.com/s{i}> <https://example.com/p> "{i}" .' for i in range(10)
    )
    it = parse(data, "turtle")
    assert next(it).object.value == "0"
    del it


def test_roundtrip_path(tmp_path: Path) -> None:
    model = Model()
    model.add(
        Triple(
            NamedNode("https://example.com/alice"),
            NamedNode("https://example.com/name"),
            Literal("Alice"),
        )
    )
    path = tmp_path / "graph.ttl"
    serialize_path(model, path, "turtle")
    loaded = Model()
    from oxiland import load_path

    assert load_path(loaded, path) == 1
    assert len(loaded) == 1


def test_unsupported_syntax() -> None:
    with pytest.raises(UnsupportedError):
        Syntax.from_name("json-ld")
