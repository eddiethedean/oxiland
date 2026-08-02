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
ALLOWED_MILESTONES = {
    "0.1",
    "0.2",
    "0.3",
    "0.4",
    "0.5",
    "0.6",
    "0.7",
    "0.8",
    "0.9",
    "0.10",
    "0.11",
}
C_ABI_REQUIRED_FROM = "0.8"


def milestone_key(value: str) -> tuple[int, ...]:
    """Returns a numeric milestone key (``0.10`` must sort after ``0.9``)."""

    try:
        return tuple(int(part) for part in value.split("."))
    except ValueError:
        fail(f"invalid numeric milestone {value!r}")


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
        if entry["state"] == "excluded":
            notes = entry.get("notes")
            deviations = entry.get("deviations") or []
            if not notes and not deviations:
                fail(
                    f"{path.name}:{entry['id']}: excluded entries require notes or deviations"
                )

        if milestone_key(milestone) >= milestone_key(C_ABI_REQUIRED_FROM):
            if "c_abi" not in entry:
                fail(f"{path.name}:{entry['id']}: missing c_abi (required from 0.8)")
            if "c_state" not in entry:
                fail(f"{path.name}:{entry['id']}: missing c_state (required from 0.8)")
            if entry["c_state"] not in ALLOWED_STATES:
                fail(f"{path.name}:{entry['id']}: invalid c_state {entry['c_state']}")
            if entry["c_state"] in {"implemented", "verified"}:
                if not entry.get("c_abi"):
                    fail(
                        f"{path.name}:{entry['id']}: c_abi required when c_state is "
                        f"{entry['c_state']}"
                    )
                c_tests = entry.get("c_tests")
                if not isinstance(c_tests, list) or not c_tests:
                    fail(
                        f"{path.name}:{entry['id']}: c_tests required when c_state is "
                        f"{entry['c_state']}"
                    )

    if milestone == "0.6":
        unreviewed = [e["id"] for e in entries if e["state"] == "unreviewed"]
        if unreviewed:
            fail(
                f"{path.name}: 0.6 forbids unreviewed entries "
                f"({len(unreviewed)} remain), e.g. {unreviewed[:5]}"
            )
        mapped = [e["id"] for e in entries if e["state"] == "mapped"]
        if mapped:
            fail(
                f"{path.name}: 0.6 forbids mapped (unimplemented) entries "
                f"({len(mapped)} remain), e.g. {mapped[:5]}"
            )

    if milestone == "0.11":
        for entry in entries:
            obligations = entry.get("obligations")
            if not isinstance(obligations, list) or not obligations:
                fail(f"{path.name}:{entry['id']}: 0.11 requires non-empty obligations")
            if entry["state"] == "excluded":
                fail(f"{path.name}:{entry['id']}: 0.11 forbids excluded entries")
            if entry.get("deviations"):
                fail(f"{path.name}:{entry['id']}: 0.11 forbids deviations")

    counts = Counter(entry["state"] for entry in entries)
    print(f"inventory ok: {path.relative_to(ROOT)}")
    print(f"entries: {len(entries)}")
    for state in sorted(counts):
        print(f"  {state}: {counts[state]}")
    if milestone_key(milestone) >= milestone_key(C_ABI_REQUIRED_FROM):
        c_counts = Counter(entry.get("c_state", "missing") for entry in entries)
        print("c_state:")
        for state in sorted(c_counts):
            print(f"  {state}: {c_counts[state]}")


def main() -> None:
    inventories = sorted(INVENTORY_DIR.glob("redland-*.json"))
    if not inventories:
        fail(f"no inventories found in {INVENTORY_DIR}")
    for path in inventories:
        validate_inventory(path)


if __name__ == "__main__":
    main()
