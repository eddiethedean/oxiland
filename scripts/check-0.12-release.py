#!/usr/bin/env python3
"""Fail-closed qualification checker for the 0.12 performance milestone.

The checker deliberately separates arithmetic from release provenance. It
reuses ``check-performance-gate.py`` for per-case statistics, then requires a
frozen checksummed protocol, exact native profile coverage, candidate-bound
artifacts, and refreshed 0.11 parity evidence for the same Git revision.
"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "compatibility" / "qualification" / "0.12-matrix.json"
HEX_SHA256 = re.compile(r"[0-9a-f]{64}")


def fail(message: str) -> None:
    raise ValueError(message)


def load_json(path: Path) -> dict[str, Any]:
    if not path.is_file():
        fail(f"missing required artifact: {path.relative_to(ROOT)}")
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        fail(f"artifact is not a JSON object: {path.relative_to(ROOT)}")
    return value


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_script(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        fail(f"cannot load checker: {path.relative_to(ROOT)}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def require_digest(value: Any, field: str) -> str:
    if not isinstance(value, str) or HEX_SHA256.fullmatch(value) is None:
        fail(f"{field} must be a lowercase SHA-256 digest")
    return value


def resolve_repo_path(value: Any, field: str) -> Path:
    if not isinstance(value, str) or not value:
        fail(f"{field} must be a repository-relative path")
    path = Path(value)
    if path.is_absolute() or ".." in path.parts:
        fail(f"{field} must stay within the repository")
    return ROOT / path


def validate_frozen_protocol(
    matrix: dict[str, Any], suite: dict[str, Any]
) -> tuple[Path, list[str]]:
    if matrix.get("schema_version") != 1 or matrix.get("milestone") != "0.12":
        fail("0.12 matrix has the wrong schema or milestone")
    if matrix.get("status") != "frozen":
        fail("0.12 matrix is not frozen")
    if suite.get("schema_version") != 1 or suite.get("milestone") != "0.12":
        fail("0.12 suite has the wrong schema or milestone")
    if suite.get("status") != "frozen":
        fail("0.12 performance suite is not frozen")

    performance = matrix.get("performance")
    if not isinstance(performance, dict):
        fail("0.12 matrix is missing performance configuration")
    suite_path = resolve_repo_path(performance.get("suite"), "performance.suite")
    harness_path = resolve_repo_path(performance.get("harness"), "performance.harness")
    if not suite_path.is_file() or not harness_path.is_file():
        fail("checksummed performance suite or harness is missing")
    if sha256_file(suite_path) != require_digest(
        performance.get("suite_sha256"), "performance.suite_sha256"
    ):
        fail("0.12 suite checksum mismatch")
    if sha256_file(harness_path) != require_digest(
        performance.get("harness_sha256"), "performance.harness_sha256"
    ):
        fail("0.12 harness checksum mismatch")

    cases = suite.get("cases")
    if not isinstance(cases, list) or not cases:
        fail("0.12 suite has no cases")
    case_ids = [item.get("id") for item in cases if isinstance(item, dict)]
    if len(case_ids) != len(cases) or len(set(case_ids)) != len(case_ids):
        fail("0.12 suite case IDs must be present and unique")
    if any(item.get("required") is not True for item in cases):
        fail("every 0.12 performance case must be required")

    budgets = suite.get("resource_budgets")
    if not isinstance(budgets, list) or not budgets:
        fail("0.12 resource budgets are not frozen")
    budget_ids: list[str] = []
    for budget in budgets:
        if not isinstance(budget, dict) or not isinstance(budget.get("id"), str):
            fail("0.12 resource budget is missing an ID")
        budget_ids.append(budget["id"])
        maximum = budget.get("maximum")
        if not isinstance(maximum, (int, float)) or maximum <= 0:
            fail(f"resource budget {budget['id']} has no positive maximum")
    if len(set(budget_ids)) != len(budget_ids):
        fail("0.12 resource budget IDs must be unique")

    profiles = performance.get("profiles")
    if not isinstance(profiles, list) or not profiles:
        fail("0.12 performance profile matrix is empty")
    if not all(isinstance(profile, str) and "/" in profile for profile in profiles):
        fail("0.12 performance profile IDs are malformed")
    if len(set(profiles)) != len(profiles):
        fail("0.12 performance profiles must be unique")
    expected_targets = set(matrix.get("targets") or [])
    actual_targets = {profile.split("/", 1)[0] for profile in profiles}
    if actual_targets != expected_targets:
        fail("0.12 performance profiles do not cover exactly the frozen targets")

    evidence_dir = resolve_repo_path(
        performance.get("evidence_dir"), "performance.evidence_dir"
    )
    return evidence_dir, profiles


def validate_performance_bundle(
    matrix: dict[str, Any],
    suite: dict[str, Any],
    candidate_revision: str,
    performance_gate: Any,
) -> list[dict[str, Any]]:
    evidence_dir, profiles = validate_frozen_protocol(matrix, suite)
    reports: list[dict[str, Any]] = []
    evidence_revisions: set[str] = set()
    execution_ids: set[str] = set()

    for profile_id in profiles:
        path = evidence_dir / f"{profile_id.replace('/', '__')}.json"
        data = load_json(path)
        target, profile = profile_id.split("/", 1)
        if data.get("target") != target or data.get("profile") != profile:
            fail(f"{profile_id}: target/profile provenance mismatch")
        if data.get("synthetic") is not False:
            fail(f"{profile_id}: evidence must explicitly set synthetic=false")
        if data.get("clean_worktree") is not True:
            fail(f"{profile_id}: evidence was not measured from a clean worktree")
        if data.get("git_revision") != candidate_revision:
            fail(f"{profile_id}: evidence is stale or from another candidate")

        evidence_revision = data.get("evidence_revision")
        execution_id = data.get("execution_id")
        if not isinstance(evidence_revision, str) or not evidence_revision:
            fail(f"{profile_id}: missing evidence_revision")
        if evidence_revision in evidence_revisions:
            fail(f"{profile_id}: duplicate evidence_revision (profile fan-out)")
        evidence_revisions.add(evidence_revision)
        if not isinstance(execution_id, str) or len(execution_id) < 16:
            fail(f"{profile_id}: missing independent execution_id")
        if execution_id in execution_ids:
            fail(f"{profile_id}: duplicate execution_id (profile fan-out)")
        execution_ids.add(execution_id)

        artifacts = data.get("artifacts")
        if not isinstance(artifacts, dict):
            fail(f"{profile_id}: missing artifact hashes")
        for name in ("oxiland_library_sha256", "redland_library_sha256", "harness_sha256"):
            require_digest(artifacts.get(name), f"{profile_id}.artifacts.{name}")
        if artifacts["harness_sha256"] != matrix["performance"]["harness_sha256"]:
            fail(f"{profile_id}: benchmark harness hash mismatch")

        report = performance_gate.evaluate(data, suite)
        if report.get("passed") is not True:
            failed_cases = [case["id"] for case in report["cases"] if not case["passed"]]
            failed_resources = [
                item["id"] for item in report["resource_checks"] if not item["passed"]
            ]
            fail(f"{profile_id}: performance gate failed: {failed_cases + failed_resources}")
        reports.append(report)
    return reports


def git_head() -> str:
    return subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
    ).strip()


def require_clean_worktree() -> None:
    status = subprocess.check_output(
        ["git", "status", "--porcelain", "--untracked-files=all"],
        cwd=ROOT,
        text=True,
    )
    if status.strip():
        fail("release qualification requires a clean worktree")


def evaluate() -> dict[str, Any]:
    matrix = load_json(MATRIX)
    performance = matrix.get("performance")
    if not isinstance(performance, dict):
        fail("0.12 matrix is missing performance configuration")
    suite_path = resolve_repo_path(performance.get("suite"), "performance.suite")
    suite = load_json(suite_path)
    validate_frozen_protocol(matrix, suite)

    parity = matrix.get("parity")
    if not isinstance(parity, dict):
        fail("0.12 matrix is missing parity retention configuration")
    parity_evidence = load_json(
        resolve_repo_path(parity.get("evidence"), "parity.evidence")
    )
    candidate_revision = parity_evidence.get("git_revision")
    if not isinstance(candidate_revision, str) or not candidate_revision:
        fail("parity evidence is not revision-bound")
    if candidate_revision != git_head():
        fail("0.11 parity evidence is stale for the 0.12 candidate")
    require_clean_worktree()

    parity_checker = load_script(
        "check_011_for_012",
        resolve_repo_path(parity.get("checker"), "parity.checker"),
    )
    parity_report = parity_checker.evaluate()
    if parity_report.get("passed") is not True:
        fail("0.11 parity retention gate did not pass")

    performance_gate = load_script(
        "performance_gate_for_012", ROOT / "scripts" / "check-performance-gate.py"
    )
    reports = validate_performance_bundle(
        matrix, suite, candidate_revision, performance_gate
    )
    return {
        "passed": True,
        "candidate_revision": candidate_revision,
        "performance_profiles": len(reports),
        "performance_cases": sum(len(report["cases"]) for report in reports),
        "resource_checks": sum(len(report["resource_checks"]) for report in reports),
        "parity_profiles": len(parity_report["profiles"]),
    }


def main() -> int:
    try:
        report = evaluate()
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"0.12 release gate failed: {error}", file=sys.stderr)
        return 1
    print("0.12 release qualification passes")
    print(
        f"revision={report['candidate_revision']} "
        f"performance_profiles={report['performance_profiles']} "
        f"performance_cases={report['performance_cases']} "
        f"resource_checks={report['resource_checks']} "
        f"parity_profiles={report['parity_profiles']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
