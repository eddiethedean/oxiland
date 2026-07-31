#!/usr/bin/env python3
"""Validate Oxiland compatibility inventory manifests."""

from __future__ import annotations

import json
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INVENTORY_DIR = ROOT / "compatibility" / "inventory"
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
ALLOWED_MILESTONES = {"0.1", "0.2", "0.3", "0.4"}


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def validate_inventory(path: Path) -> None:
    data = json.loads(path.read_text(encoding="utf-8"))
    if data.get("schema_version") != 1:
        fail(f"{path.name}: unsupported schema_version")
    milestone = data.get("milestone")
    if milestone not in ALLOWED_MILESTONES:
        fail(f"{path.name}: unexpected milestone {milestone!r}")

    entries = data.get("entries")
    if not isinstance(entries, list) or not entries:
        fail(f"{path.name}: entries must be a non-empty list")

    ids: set[str] = set()
    for entry in entries:
        missing = REQUIRED_FIELDS - set(entry)
        if missing:
            fail(f"{path.name}:{entry.get('id', '<unknown>')}: missing fields {sorted(missing)}")
        if entry["id"] in ids:
            fail(f"{path.name}: duplicate id {entry['id']}")
        ids.add(entry["id"])
        if entry["state"] not in ALLOWED_STATES:
            fail(f"{path.name}:{entry['id']}: invalid state {entry['state']}")

        impl = ROOT / entry["implementation"]
        if not impl.is_file():
            fail(
                f"{path.name}:{entry['id']}: implementation path missing: {entry['implementation']}"
            )

        if not isinstance(entry["tests"], list) or not entry["tests"]:
            fail(f"{path.name}:{entry['id']}: tests must be a non-empty list")
        for test_ref in entry["tests"]:
            if not isinstance(test_ref, str) or not test_ref.strip():
                fail(f"{path.name}:{entry['id']}: empty test reference")
            path_part, _, fn_part = test_ref.partition("::")
            test_path = ROOT / path_part
            if not test_path.is_file():
                fail(f"{path.name}:{entry['id']}: test path missing: {path_part}")
            if fn_part:
                text = test_path.read_text(encoding="utf-8")
                if f"fn {fn_part}(" not in text and f"fn {fn_part}<" not in text:
                    fail(
                        f"{path.name}:{entry['id']}: test function missing: "
                        f"{test_ref}"
                    )

        # Optional 0.2 metadata: when present, must be well-formed.
        if "fixtures" in entry:
            if not isinstance(entry["fixtures"], list):
                fail(f"{path.name}:{entry['id']}: fixtures must be a list")
            for fixture in entry["fixtures"]:
                fixture_path = ROOT / fixture
                if not fixture_path.exists():
                    fail(
                        f"{path.name}:{entry['id']}: fixture path missing: {fixture}"
                    )
        if "deviations" in entry and not isinstance(entry["deviations"], list):
            fail(f"{path.name}:{entry['id']}: deviations must be a list")

    counts = Counter(entry["state"] for entry in entries)
    print(f"inventory ok: {path.relative_to(ROOT)}")
    print(f"entries: {len(entries)}")
    for state in sorted(counts):
        print(f"  {state}: {counts[state]}")


def main() -> None:
    inventories = sorted(INVENTORY_DIR.glob("redland-*.json"))
    if not inventories:
        fail(f"no inventories found in {INVENTORY_DIR}")
    for path in inventories:
        validate_inventory(path)


if __name__ == "__main__":
    main()
