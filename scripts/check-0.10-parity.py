#!/usr/bin/env python3
"""Generate and enforce the Oxiland 0.10 full-Redland parity report."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def load_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"{path}: root must be an object")
    return value


def evaluate(inventory_path: Path, evidence_path: Path) -> dict[str, Any]:
    inventory = load_object(inventory_path)
    evidence = load_object(evidence_path)
    if inventory.get("milestone") != "0.10":
        raise ValueError("inventory milestone must be 0.10")
    if inventory.get("redland_api") != "1.0.17":
        raise ValueError("inventory must pin Redland 1.0.17")
    entries = inventory.get("entries")
    if not isinstance(entries, list) or not entries:
        raise ValueError("inventory entries must be non-empty")

    public = [entry for entry in entries if str(entry.get("symbol", "")).startswith("librdf_")]
    symbols = [entry["symbol"] for entry in public]
    if len(symbols) != len(set(symbols)):
        raise ValueError("inventory contains duplicate public symbols")

    inventory_failures: list[dict[str, str]] = []
    safe_verified = 0
    c_verified = 0
    for entry in public:
        symbol = entry["symbol"]
        state = entry.get("state")
        c_state = entry.get("c_state")
        if state == "verified":
            safe_verified += 1
        elif state == "not-applicable" and entry.get("safe_n_a_kind") == "ownership-mechanic":
            # Manual ownership may be N/A only in safe Rust. Its C form remains
            # mandatory below, exactly as the compatibility contract requires.
            pass
        else:
            inventory_failures.append({"symbol": symbol, "reason": f"safe state is {state!r}"})
        if c_state == "verified":
            c_verified += 1
        else:
            inventory_failures.append({"symbol": symbol, "reason": f"C state is {c_state!r}"})
        if entry.get("deviations"):
            inventory_failures.append({"symbol": symbol, "reason": "has behavioral deviation"})

    profiles = evidence.get("profiles")
    if evidence.get("schema_version") != 1 or not isinstance(profiles, list) or not profiles:
        raise ValueError("evidence requires schema_version 1 and non-empty profiles")
    expected_profiles = evidence.get("expected_profiles")
    if not isinstance(expected_profiles, list) or not expected_profiles:
        raise ValueError("evidence requires frozen expected_profiles")

    profile_reports: list[dict[str, Any]] = []
    seen_profiles: set[str] = set()
    symbol_set = set(symbols)
    for profile in profiles:
        profile_id = profile.get("id")
        if not isinstance(profile_id, str) or not profile_id:
            raise ValueError("profile missing id")
        if profile_id in seen_profiles:
            raise ValueError(f"duplicate profile {profile_id}")
        seen_profiles.add(profile_id)
        verified = profile.get("verified_symbols")
        if not isinstance(verified, list):
            raise ValueError(f"{profile_id}: verified_symbols must be a list")
        verified_set = set(verified)
        unknown = sorted(verified_set - symbol_set)
        missing = sorted(symbol_set - verified_set)
        skips = profile.get("skips")
        mismatches = profile.get("mismatches")
        quarantined = profile.get("quarantined")
        deviations = profile.get("deviations")
        clean = all(value == [] for value in (skips, mismatches, quarantined, deviations))
        passed = not unknown and not missing and clean and profile.get("differential_passed") is True
        profile_reports.append(
            {
                "id": profile_id,
                "target": profile.get("target"),
                "build_profile": profile.get("build_profile"),
                "oracle": profile.get("oracle"),
                "evidence_revision": profile.get("evidence_revision"),
                "numerator": len(verified_set & symbol_set),
                "denominator": len(symbols),
                "skip_count": len(skips) if isinstance(skips, list) else -1,
                "missing": missing,
                "unknown": unknown,
                "passed": passed,
            }
        )

    matrix_complete = seen_profiles == set(expected_profiles)
    passed = not inventory_failures and matrix_complete and all(item["passed"] for item in profile_reports)
    return {
        "schema_version": 1,
        "baseline": "Redland librdf 1.0.17 (manual 1.0.18)",
        "inventory": str(inventory_path),
        "inventory_sha256": sha256(inventory_path),
        "evidence": str(evidence_path),
        "evidence_sha256": sha256(evidence_path),
        "public_symbol_denominator": len(symbols),
        "safe_verified": safe_verified,
        "safe_not_applicable": sum(entry.get("state") == "not-applicable" for entry in public),
        "c_verified": c_verified,
        "inventory_failures": inventory_failures,
        "expected_profiles": expected_profiles,
        "matrix_complete": matrix_complete,
        "profiles": profile_reports,
        "passed": passed,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("inventory", type=Path)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    try:
        report = evaluate(args.inventory, args.evidence)
    except (OSError, json.JSONDecodeError, KeyError, TypeError, ValueError) as error:
        print(f"parity gate error: {error}", file=sys.stderr)
        return 2
    encoded = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(encoded, encoding="utf-8")
    else:
        print(encoded, end="")
    if not report["passed"]:
        print("0.10 full-Redland parity gate failed", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
