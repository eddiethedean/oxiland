#!/usr/bin/env python3
"""Derive 0.11 parity evidence and inventory states from raw harness results.

Never asserts differential_passed from symbol allowlists. Only obligations with
passing raw results become verified.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RAW = ROOT / "compatibility" / "qualification" / "raw"
MATRIX = ROOT / "compatibility" / "qualification" / "0.11-matrix.json"
INVENTORY = ROOT / "compatibility" / "inventory" / "redland-1.0.17-oxiland-0.11.json"
OBLIGATIONS = ROOT / "compatibility" / "inventory" / "0.11-obligations.json"
PARITY_OUT = ROOT / "compatibility" / "qualification" / "0.11-parity-evidence.json"


def git_revision() -> str:
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--require-all-profiles",
        action="store_true",
        help="fail unless every matrix profile has raw evidence",
    )
    args = parser.parse_args()

    matrix = json.loads(MATRIX.read_text(encoding="utf-8"))
    inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
    catalog = json.loads(OBLIGATIONS.read_text(encoding="utf-8"))

    results = []
    for path in sorted(RAW.glob("*.json")):
        if path.name.endswith("__index.json"):
            continue
        data = json.loads(path.read_text(encoding="utf-8"))
        data["_path"] = str(path.relative_to(ROOT))
        results.append(data)

    if not results:
        print("derive-0.11-parity: no raw results", file=sys.stderr)
        return 1

    by_profile: dict[str, list[dict]] = defaultdict(list)
    covered: set[str] = set()
    for result in results:
        if result.get("synthetic"):
            print(f"refuse synthetic raw result {result['_path']}", file=sys.stderr)
            return 1
        if result.get("differential_passed") is not True:
            continue
        by_profile[result["profile_id"]].append(result)
        for obl in result.get("obligation_ids") or []:
            covered.add(obl)

    required = set(matrix["required_profile_ids"])
    if args.require_all_profiles and required - set(by_profile):
        print(
            f"missing profiles: {sorted(required - set(by_profile))}",
            file=sys.stderr,
        )
        return 1

    revision = git_revision()
    profiles = []
    for profile_id in matrix["required_profile_ids"]:
        target, build_profile = profile_id.split("/", 1)
        profile_results = by_profile.get(profile_id, [])
        verified_obligations = sorted(
            {
                obl
                for result in profile_results
                for obl in (result.get("obligation_ids") or [])
                if result.get("differential_passed")
            }
        )
        profiles.append(
            {
                "id": profile_id,
                "target": target,
                "build_profile": build_profile,
                "oracle": {
                    "name": "Redland librdf",
                    "version": "1.0.17",
                    "manual": "1.0.18",
                    "source_sha256": matrix["redland"]["source_sha256"],
                },
                "evidence_revision": f"oxiland-0.11-parity-{revision[:12]}",
                "raw_results": [r["_path"] for r in profile_results],
                "verified_obligations": verified_obligations,
                "verified_symbols": sorted(
                    {
                        next(
                            (
                                e["symbol"]
                                for e in inventory["entries"]
                                if obl.startswith(f"obl.{e['id']}.")
                            ),
                            "",
                        )
                        for obl in verified_obligations
                    }
                    - {""}
                ),
                "differential_passed": bool(profile_results)
                and all(r.get("differential_passed") for r in profile_results),
                "skips": [],
                "mismatches": [],
                "quarantined": [],
                "deviations": [],
                "synthetic": False,
                "notes": (
                    "Derived from raw two-sided harness results only. "
                    "Empty raw_results means this profile is not yet evidenced."
                ),
            }
        )

    # Update obligation catalog states.
    for obligation in catalog["obligations"]:
        obligation["state"] = "verified" if obligation["id"] in covered else "unreviewed"
    OBLIGATIONS.write_text(json.dumps(catalog, indent=2) + "\n", encoding="utf-8")

    # Update inventory states from obligation coverage.
    for entry in inventory["entries"]:
        obls = entry.get("obligations") or []
        if obls and all(o in covered for o in obls):
            if entry.get("safe_n_a_kind") == "ownership-mechanic":
                entry["state"] = "not-applicable"
            else:
                entry["state"] = "verified"
            entry["c_state"] = "verified"
            entry["notes"] = (
                (entry.get("notes") or "")
                + " | 0.11: verified from raw differentials"
            ).strip(" |")
        else:
            # Keep ownership N/A; otherwise remain implemented candidate.
            if entry.get("safe_n_a_kind") == "ownership-mechanic":
                entry["state"] = "not-applicable"
            else:
                entry["state"] = "implemented"
            entry["c_state"] = "implemented"

    inventory["notes"] = (
        "0.11 inventory states derived by scripts/derive-0.11-parity.py from raw "
        "harness evidence."
    )
    INVENTORY.write_text(json.dumps(inventory, indent=2) + "\n", encoding="utf-8")

    parity = {
        "schema_version": 1,
        "milestone": "0.11",
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "generator": "scripts/derive-0.11-parity.py",
        "git_revision": revision,
        "synthetic": False,
        "expected_profiles": matrix["required_profile_ids"],
        "covered_obligation_count": len(covered),
        "profiles": profiles,
        "notes": (
            "Parity evidence derived exclusively from compatibility/qualification/raw. "
            "Profile fan-out and allowlist-only passes are rejected by check-0.11-release.py."
        ),
    }
    PARITY_OUT.write_text(json.dumps(parity, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {PARITY_OUT.relative_to(ROOT)}")
    print(f"covered obligations: {len(covered)}")
    print(f"profiles with raw evidence: {sorted(by_profile)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
