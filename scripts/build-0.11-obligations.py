#!/usr/bin/env python3
"""Generate the 0.11 behavior-obligation catalog from the candidate inventory.

Every public inventory row receives positive, boundary, failure, and lifecycle
obligations. Categories map only to fixtures that exercise that category;
shallow positive fixtures no longer auto-cover failure/boundary.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "compatibility/inventory/redland-1.0.17-oxiland-0.10.json"
OUT = ROOT / "compatibility/inventory/0.11-obligations.json"
BASELINE = ROOT / "compatibility/baseline/0.11-baseline-manifest.json"

CATEGORIES = ("positive", "boundary", "failure", "lifecycle")

# Subsystem → shared fixture covering the family (path relative to repo root).
POSITIVE_FIXTURES = {
    "init": "compatibility/fixtures/0.11/world-lifecycle.json",
    "world": "compatibility/fixtures/0.11/world-lifecycle.json",
    "uri": "compatibility/fixtures/0.11/uri-node.json",
    "node": "compatibility/fixtures/0.11/uri-node.json",
    "statement": "compatibility/fixtures/0.11/statement-model.json",
    "model": "compatibility/fixtures/0.11/statement-model.json",
    "storage": "compatibility/fixtures/0.11/storage-memory.json",
    "stream": "compatibility/fixtures/0.11/stream-find.json",
    "iterator": "compatibility/fixtures/0.11/stream-find.json",
    "parser": "compatibility/fixtures/0.11/parse-turtle.json",
    "serializer": "compatibility/fixtures/0.11/serialize-ntriples.json",
    "query": "compatibility/fixtures/0.11/sparql-ask-select.json",
    "query_results": "compatibility/fixtures/0.11/sparql-ask-select.json",
    "digest": "compatibility/fixtures/0.11/digest-hash.json",
    "hash": "compatibility/fixtures/0.11/digest-hash.json",
    "list": "compatibility/fixtures/0.11/list-lifecycle.json",
    "log": "compatibility/fixtures/0.11/logging-callback.json",
    "concepts": "compatibility/fixtures/0.11/concepts.json",
    "files": "compatibility/fixtures/0.11/files-heuristics.json",
    "heuristics": "compatibility/fixtures/0.11/files-heuristics.json",
    "utf8": "compatibility/fixtures/0.11/unicode.json",
    "raptor": "compatibility/fixtures/0.11/world-lifecycle.json",
    "utility": "compatibility/fixtures/0.11/cli-parse-ask.json",
}

BOUNDARY_FIXTURE = "compatibility/fixtures/0.11/model-boundary-empty.json"
FAILURE_FIXTURE = "compatibility/fixtures/0.11/parse-turtle-failure.json"
LIFECYCLE_FIXTURE = "compatibility/fixtures/0.11/world-lifecycle.json"


def fixture_for(subsystem: str, category: str) -> str:
    if category == "failure":
        return FAILURE_FIXTURE
    if category == "boundary":
        return BOUNDARY_FIXTURE
    if category == "lifecycle":
        return LIFECYCLE_FIXTURE
    return POSITIVE_FIXTURES.get(subsystem, LIFECYCLE_FIXTURE)


def observations_for(category: str) -> list[str]:
    base = ["return", "error"]
    if category == "positive":
        return base + ["rdf", "bytes"]
    if category == "boundary":
        return base + ["rdf"]
    if category == "failure":
        return base + ["diagnostics"]
    return base + ["ownership", "callback", "persist"]


def main() -> int:
    inv = json.loads(INVENTORY.read_text(encoding="utf-8"))
    baseline_sha = None
    if BASELINE.is_file():
        baseline_sha = hashlib.sha256(BASELINE.read_bytes()).hexdigest()

    obligations: list[dict] = []
    for entry in inv["entries"]:
        inv_id = entry["id"]
        symbol = entry["symbol"]
        subsystem = entry.get("subsystem") or "utility"
        for category in CATEGORIES:
            obl_id = f"obl.{inv_id}.{category}"
            obligations.append(
                {
                    "id": obl_id,
                    "inventory_ids": [inv_id],
                    "symbol": symbol,
                    "category": category,
                    "fixture": fixture_for(subsystem, category),
                    "normalization_profile": (
                        "diagnostics"
                        if category == "failure"
                        else "rdf-dataset"
                        if category in {"positive", "boundary"}
                        else "lifecycle"
                    ),
                    "observations": observations_for(category),
                    "state": "unreviewed",
                }
            )

    catalog = {
        "schema_version": 1,
        "milestone": "0.11",
        "generated_by": "scripts/build-0.11-obligations.py",
        "baseline_manifest_sha256": baseline_sha,
        "source_inventory": str(INVENTORY.relative_to(ROOT)),
        "obligation_count": len(obligations),
        "categories": list(CATEGORIES),
        "notes": (
            "Every public inventory row links four obligations. Failure and "
            "boundary map to dedicated fixtures; positive maps to subsystem "
            "fixtures; lifecycle maps to world open/close. States start as "
            "unreviewed and may only become verified from raw two-sided C "
            "oracle harness results."
        ),
        "obligations": obligations,
    }
    OUT.write_text(json.dumps(catalog, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    print(f"obligations: {len(obligations)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
