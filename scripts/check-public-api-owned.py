#!/usr/bin/env python3
"""Validate the intentional public API baseline against owned crate sources.

The baseline in api/oxiland-public-api.txt is a curated allowlist. This script
ensures Oxiland-owned public items discovered under src/ match that allowlist,
so CI fails when owned surface is added or removed without updating the snapshot.

Oxigraph re-exports under terms / sparql / io::primitives are represented only as
modules (plus terms::named_node / blank_node) and are not exhaustively listed.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "api" / "oxiland-public-api.txt"

# Only match truly public items (not pub(crate) / pub(super)).
PUB_ITEM = re.compile(
    r"^(?P<indent> *)pub +(?:const +)?(?P<kind>fn|struct|enum|type|const|static|trait) +(?P<name>\w+)"
)
IMPL_HEAD = re.compile(
    r"^(?P<indent> *)impl(?:<[^>]*>)?\s+(?:(?P<trait>.+?)\s+for\s+)?(?P<type>[\w:]+)"
)
PUB_FIELD = re.compile(r"^(?P<indent> *)pub +(?P<name>\w+)\s*:")
VARIANT = re.compile(r"^(?P<indent> +)(?P<name>[A-Z]\w*)\s*(?:\{|\(|,|$)")

MODULES = {
    ROOT / "src" / "error.rs": "oxiland",
    ROOT / "src" / "model.rs": "oxiland",
    ROOT / "src" / "query.rs": "oxiland",
    ROOT / "src" / "storage.rs": "oxiland",
    ROOT / "src" / "world.rs": "oxiland",
    ROOT / "src" / "io" / "format.rs": "oxiland::io",
    ROOT / "src" / "io" / "parser.rs": "oxiland::io",
    ROOT / "src" / "io" / "serializer.rs": "oxiland::io",
    ROOT / "src" / "io" / "location.rs": "oxiland::io",
}


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def load_baseline() -> set[str]:
    return {
        line.strip()
        for line in BASELINE.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.strip().startswith("#")
    }


def parse_file(path: Path, prefix: str) -> set[str]:
    items: set[str] = set()
    lines = path.read_text(encoding="utf-8").splitlines()
    impl_type: str | None = None
    impl_indent: int | None = None
    i = 0
    while i < len(lines):
        raw = lines[i]
        line = raw.split("//", 1)[0].rstrip()
        if not line.strip():
            i += 1
            continue

        indent = len(line) - len(line.lstrip(" "))

        if impl_type is not None and impl_indent is not None and indent <= impl_indent:
            if line.strip() == "}":
                impl_type = None
                impl_indent = None
                i += 1
                continue

        impl_match = IMPL_HEAD.match(line)
        if impl_match and line.rstrip().endswith("{") and impl_match.group("trait") is None:
            # Inherent impl only (skip trait impls).
            impl_type = impl_match.group("type").split("::")[-1]
            impl_indent = indent
            i += 1
            continue

        item = PUB_ITEM.match(line)
        if item:
            kind = item.group("kind")
            name = item.group("name")
            item_indent = len(item.group("indent"))

            if impl_type is not None and kind == "fn" and item_indent > (impl_indent or 0):
                items.add(f"{prefix}::{impl_type}::{name}")
                i += 1
                continue

            if item_indent == 0:
                full = f"{prefix}::{name}"
                items.add(full)
                if kind == "enum":
                    i += 1
                    while i < len(lines):
                        nested = lines[i].split("//", 1)[0].rstrip()
                        if nested.strip() == "}":
                            break
                        variant = VARIANT.match(nested)
                        if variant:
                            items.add(f"{full}::{variant.group('name')}")
                        i += 1
                i += 1
                continue

        field = PUB_FIELD.match(line)
        if field and impl_type is None:
            # Public struct fields (StatementPattern).
            # Attach to the most recently declared struct at indent 0 — tracked below.
            pass

        i += 1

    # Second pass for public struct fields (StatementPattern).
    current_struct: str | None = None
    struct_indent: int | None = None
    for raw in lines:
        line = raw.split("//", 1)[0].rstrip()
        if not line.strip():
            continue
        indent = len(line) - len(line.lstrip(" "))
        item = PUB_ITEM.match(line)
        if item and item.group("kind") == "struct" and indent == 0:
            current_struct = item.group("name")
            struct_indent = indent
            continue
        if current_struct is not None and struct_indent is not None:
            if indent <= struct_indent and line.strip() == "}":
                current_struct = None
                struct_indent = None
                continue
            field = PUB_FIELD.match(line)
            if field and indent > struct_indent:
                items.add(f"{prefix}::{current_struct}::{field.group('name')}")

    return items


def discover_owned() -> set[str]:
    owned: set[str] = {
        "oxiland",
        "oxiland::Result",
        "oxiland::io",
        "oxiland::io::primitives",
        "oxiland::terms",
        "oxiland::sparql",
        "oxiland::storage",
        "oxiland::terms::named_node",
        "oxiland::terms::blank_node",
        "oxiland::QueryResults",  # re-exported from query module
    }
    for path, prefix in MODULES.items():
        if not path.is_file():
            fail(f"missing source file: {path.relative_to(ROOT)}")
        owned |= parse_file(path, prefix)
    return owned


def main() -> int:
    baseline = load_baseline()
    owned = discover_owned()

    # Compare only paths that the ownership scanner is responsible for.
    # Baseline may intentionally omit exhaustive oxigraph re-exports.
    missing_from_baseline = sorted(owned - baseline)
    missing_from_source = sorted(baseline - owned)

    if missing_from_baseline or missing_from_source:
        if missing_from_baseline:
            print(
                "owned public items missing from api/oxiland-public-api.txt:\n  - "
                + "\n  - ".join(missing_from_baseline),
                file=sys.stderr,
            )
        if missing_from_source:
            print(
                "baseline entries not found in owned sources:\n  - "
                + "\n  - ".join(missing_from_source),
                file=sys.stderr,
            )
        print(
            "public API ownership check failed; update api/oxiland-public-api.txt "
            "and scripts/generate-public-api.sh together",
            file=sys.stderr,
        )
        return 1

    print(
        f"public API ownership ok: {len(owned)} owned items tracked against "
        f"{len(baseline)} baseline entries"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
