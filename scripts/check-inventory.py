#!/usr/bin/env python3
"""Validate the Oxiland compatibility inventory manifest."""

from __future__ import annotations

import json
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "compatibility" / "inventory" / "redland-1.0.17-oxiland-0.1.json"
ALLOWED_STATES = {
    "unreviewed",
    "mapped",
    "implemented",
    "verified",
    "not-applicable",
    "excluded",
}
REQUIRED_FIELDS = {
    "id",
    "symbol",
    "kind",
    "subsystem",
    "safe_rust",
    "implementation",
    "tests",
    "state",
}


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def main() -> None:
    if not INVENTORY.is_file():
        fail(f"missing inventory at {INVENTORY}")

    data = json.loads(INVENTORY.read_text(encoding="utf-8"))
    if data.get("schema_version") != 1:
        fail("unsupported schema_version")
    if data.get("milestone") != "0.1":
        fail("expected milestone 0.1")

    entries = data.get("entries")
    if not isinstance(entries, list) or not entries:
        fail("entries must be a non-empty list")

    ids: set[str] = set()
    for entry in entries:
        missing = REQUIRED_FIELDS - set(entry)
        if missing:
            fail(f"{entry.get('id', '<unknown>')}: missing fields {sorted(missing)}")
        if entry["id"] in ids:
            fail(f"duplicate id {entry['id']}")
        ids.add(entry["id"])
        if entry["state"] not in ALLOWED_STATES:
            fail(f"{entry['id']}: invalid state {entry['state']}")

        impl = ROOT / entry["implementation"]
        if not impl.is_file():
            fail(f"{entry['id']}: implementation path missing: {entry['implementation']}")

        if not isinstance(entry["tests"], list) or not entry["tests"]:
            fail(f"{entry['id']}: tests must be a non-empty list")
        for test_ref in entry["tests"]:
            path_part = test_ref.split("::", 1)[0]
            test_path = ROOT / path_part
            if not test_path.is_file():
                fail(f"{entry['id']}: test path missing: {path_part}")

    counts = Counter(entry["state"] for entry in entries)
    print(f"inventory ok: {INVENTORY.relative_to(ROOT)}")
    print(f"entries: {len(entries)}")
    for state in sorted(counts):
        print(f"  {state}: {counts[state]}")


if __name__ == "__main__":
    main()
