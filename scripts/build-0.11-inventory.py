#!/usr/bin/env python3
"""Generate the Oxiland 0.11 compatibility inventory from the 0.10 candidate.

Rules:
- Carry forward ownership-mechanic not-applicable rows.
- Attach obligation IDs from 0.11-obligations.json (required).
- Reset behavioral claims to candidate (`implemented`) unless ownership N/A;
  C rows stay `implemented` until raw 0.11 evidence verifies them.
- No deviations arrays; no synthetic verified/differential_passed.
"""

from __future__ import annotations

import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "compatibility/inventory/redland-1.0.17-oxiland-0.10.json"
DST = ROOT / "compatibility/inventory/redland-1.0.17-oxiland-0.11.json"
OBLIGATIONS = ROOT / "compatibility/inventory/0.11-obligations.json"
BASELINE = ROOT / "compatibility/baseline/0.11-baseline-manifest.json"
SYMBOLS = ROOT / "crates/oxiland-capi/symbols.version"
C_TESTS = [
    "crates/oxiland-capi/tests/ffi_lifecycle.rs",
    "crates/oxiland-capi/symbols.version",
]
SAFE_TESTS = [
    "tests/accounting.rs",
    "tests/model.rs",
    "tests/features_factories.rs",
]


def load_allowlist() -> set[str]:
    return {
        line.strip().rstrip(";")
        for line in SYMBOLS.read_text(encoding="utf-8").splitlines()
        if line.strip().startswith("librdf_")
    }


def main() -> int:
    if not OBLIGATIONS.is_file():
        print("build-0.11-inventory: run build-0.11-obligations.py first", file=sys.stderr)
        return 1
    if not BASELINE.is_file():
        print("build-0.11-inventory: run build-0.11-baseline.py first", file=sys.stderr)
        return 1

    src = json.loads(SRC.read_text(encoding="utf-8"))
    obl = json.loads(OBLIGATIONS.read_text(encoding="utf-8"))
    by_inv: dict[str, list[str]] = defaultdict(list)
    for obligation in obl["obligations"]:
        for inv_id in obligation["inventory_ids"]:
            by_inv[inv_id].append(obligation["id"])

    allowlist = load_allowlist()
    public = [
        e for e in src["entries"] if str(e.get("symbol", "")).startswith("librdf_")
    ]
    missing = sorted({e["symbol"] for e in public} - allowlist)
    if missing:
        print(
            f"build-0.11-inventory: {len(missing)} public symbols missing from "
            f"symbols.version (first 20): {missing[:20]}",
            file=sys.stderr,
        )
        return 1

    for entry in src["entries"]:
        entry.pop("deviations", None)
        inv_id = entry["id"]
        obligations = by_inv.get(inv_id)
        if not obligations:
            print(f"build-0.11-inventory: no obligations for {inv_id}", file=sys.stderr)
            return 1
        entry["obligations"] = obligations

        # Candidate reverification: keep ownership N/A; demote verified → implemented.
        if entry.get("state") == "not-applicable" and entry.get("safe_n_a_kind") == "ownership-mechanic":
            entry["state"] = "not-applicable"
            entry["notes"] = (
                (entry.get("notes") or "")
                + " | 0.11: ownership-mechanic retained; awaiting C differential"
            ).strip(" |")
        elif entry.get("state") == "verified":
            entry["state"] = "implemented"
            entry["notes"] = (
                (entry.get("notes") or "")
                + " | 0.11: candidate for reverification from raw differentials"
            ).strip(" |")
        else:
            entry["state"] = "implemented"
            entry["notes"] = (
                (entry.get("notes") or "")
                + " | 0.11: candidate"
            ).strip(" |")

        if entry.get("c_state") == "verified":
            entry["c_state"] = "implemented"
        else:
            entry["c_state"] = entry.get("c_state") or "implemented"
        entry["c_abi"] = "crates/oxiland-capi"
        entry["c_tests"] = list(C_TESTS)
        entry["tests"] = list(
            dict.fromkeys((entry.get("tests") or []) + SAFE_TESTS)
        )
        entry["implementation"] = entry.get("implementation") or "src/lib.rs"

    src["milestone"] = "0.11"
    src["oxiland_version"] = "0.11.0"
    src["generated_by"] = "scripts/build-0.11-inventory.py"
    src["baseline_manifest"] = "compatibility/baseline/0.11-baseline-manifest.json"
    src["obligation_catalog"] = "compatibility/inventory/0.11-obligations.json"
    src["notes"] = (
        "0.11 inventory: obligations attached; behavioral verified/c_state verified "
        "may only be assigned from raw two-sided harness evidence. 0.10 verified "
        "labels were demoted to implemented candidates."
    )

    DST.write_text(json.dumps(src, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    states = Counter(e.get("state") for e in src["entries"])
    cstates = Counter(e.get("c_state") for e in src["entries"])
    print(f"wrote {DST.relative_to(ROOT)}")
    print(f"safe states: {dict(states)}")
    print(f"c states: {dict(cstates)}")
    print(f"entries with obligations: {sum(1 for e in src['entries'] if e.get('obligations'))}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
