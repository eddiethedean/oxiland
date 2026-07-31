#!/usr/bin/env python3
"""Fail closed unless the complete checked-in 0.10 qualification bundle passes."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]
QUALIFICATION = ROOT / "compatibility" / "qualification"


def load_script(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / filename)
    if not spec or not spec.loader:
        raise RuntimeError(f"cannot load {filename}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def require(path: Path) -> Path:
    if not path.is_file():
        raise ValueError(f"required qualification artifact is missing: {path.relative_to(ROOT)}")
    return path


def main() -> int:
    try:
        matrix_path = require(QUALIFICATION / "0.10-matrix.json")
        inventory_path = require(
            ROOT / "compatibility" / "inventory" / "redland-1.0.17-oxiland-0.10.json"
        )
        evidence_path = require(QUALIFICATION / "0.10-parity-evidence.json")
        soak_path = require(QUALIFICATION / "0.10-soak.json")

        matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
        parity = load_script("parity_gate_release", "check-0.10-parity.py")
        parity_report = parity.evaluate(inventory_path, evidence_path)
        if not parity_report["passed"]:
            raise ValueError("full-Redland parity report does not pass")
        if set(parity_report["expected_profiles"]) != set(matrix["required_profile_ids"]):
            raise ValueError("parity evidence does not match the frozen target/profile matrix")

        performance = load_script("performance_gate_release", "check-performance-gate.py")
        suite = json.loads(require(ROOT / matrix["performance"]["suite"]).read_text(encoding="utf-8"))
        for profile in matrix["performance"]["profiles"]:
            filename = profile.replace("/", "__") + ".json"
            raw_path = require(QUALIFICATION / "performance" / filename)
            report = performance.evaluate(json.loads(raw_path.read_text(encoding="utf-8")), suite)
            if not report["passed"]:
                raise ValueError(f"performance profile failed: {profile}")

        soak = json.loads(soak_path.read_text(encoding="utf-8"))
        if soak.get("completed") is not True or soak.get("abi_resets", -1) != 0:
            raise ValueError("RC soak is incomplete or contains an ABI reset")
        if soak.get("release_blockers") != []:
            raise ValueError("RC soak retains release blockers")
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"0.10 release gate failed: {error}", file=sys.stderr)
        return 1

    print("0.10 release qualification bundle passes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
