#!/usr/bin/env python3
"""Fail-closed qualification checker for the suite-wide faster-than-Redland gate.

Requires three independent corrected-runner evidence files per frozen target
(ADR-029). Reuses ``check-performance-gate.py`` for per-case statistics.
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
MATRIX = ROOT / "compatibility" / "qualification" / "0.13-matrix.json"
HEX_SHA256 = re.compile(r"[0-9a-f]{64}")
RUN_FILE = re.compile(
    r"^(?P<target>.+)__release-default__run(?P<run>\d+)\.json$"
)


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
    data = path.read_bytes()
    if path.suffix in {".c", ".h", ".json", ".md", ".py", ".txt", ".yml", ".yaml"}:
        data = data.replace(b"\r\n", b"\n")
    digest.update(data)
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


def git_head() -> str:
    return subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
    ).strip()


def git_is_ancestor(ancestor: str, descendant: str) -> bool:
    if ancestor == descendant:
        return True
    completed = subprocess.run(
        ["git", "merge-base", "--is-ancestor", ancestor, descendant],
        cwd=ROOT,
        check=False,
        capture_output=True,
    )
    return completed.returncode == 0


def validate_frozen_protocol(
    matrix: dict[str, Any], suite: dict[str, Any]
) -> tuple[Path, list[str], int]:
    if matrix.get("schema_version") != 1 or matrix.get("milestone") != "0.13":
        fail("0.13 matrix has the wrong schema or milestone")
    if matrix.get("status") != "frozen":
        fail("0.13 matrix is not frozen")
    if suite.get("schema_version") != 1 or suite.get("milestone") != "0.13":
        fail("0.13 suite has the wrong schema or milestone")
    if suite.get("status") != "frozen":
        fail("0.13 performance suite is not frozen")

    performance = matrix.get("performance")
    if not isinstance(performance, dict):
        fail("0.13 matrix is missing performance configuration")
    suite_path = resolve_repo_path(performance.get("suite"), "performance.suite")
    harness_path = resolve_repo_path(performance.get("harness"), "performance.harness")
    if not suite_path.is_file() or not harness_path.is_file():
        fail("checksummed performance suite or harness is missing")
    if sha256_file(suite_path) != require_digest(
        performance.get("suite_sha256"), "performance.suite_sha256"
    ):
        fail("0.13 suite checksum mismatch")
    if sha256_file(harness_path) != require_digest(
        performance.get("harness_sha256"), "performance.harness_sha256"
    ):
        fail("0.13 harness checksum mismatch")

    cases = suite.get("cases")
    if not isinstance(cases, list) or not cases:
        fail("0.13 suite has no cases")
    case_ids = [item.get("id") for item in cases if isinstance(item, dict)]
    if len(case_ids) != len(cases) or len(set(case_ids)) != len(case_ids):
        fail("0.13 suite case IDs must be present and unique")
    if any(item.get("required") is not True for item in cases):
        fail("every 0.13 performance case must be required")

    budgets = suite.get("resource_budgets")
    if not isinstance(budgets, list) or not budgets:
        fail("0.13 resource budgets are not frozen")

    independent_runs = performance.get("independent_runs")
    if not isinstance(independent_runs, int) or independent_runs < 1:
        fail("0.13 matrix independent_runs must be a positive integer")
    suite_runs = suite.get("thresholds", {}).get("independent_host_runs")
    if suite_runs is not None and int(suite_runs) != independent_runs:
        fail("0.13 matrix independent_runs disagrees with the suite")

    profiles = performance.get("profiles")
    if not isinstance(profiles, list) or not profiles:
        fail("0.13 performance profile matrix is empty")
    if not all(isinstance(profile, str) and "/" in profile for profile in profiles):
        fail("0.13 performance profile IDs are malformed")
    if len(set(profiles)) != len(profiles):
        fail("0.13 performance profiles must be unique")
    expected_targets = set(matrix.get("targets") or [])
    actual_targets = {profile.split("/", 1)[0] for profile in profiles}
    if actual_targets != expected_targets:
        fail("0.13 performance profiles do not cover exactly the frozen targets")

    evidence_dir = resolve_repo_path(
        performance.get("evidence_dir"), "performance.evidence_dir"
    )
    return evidence_dir, list(profiles), independent_runs


def evidence_files_for_target(evidence_dir: Path, target: str) -> list[tuple[int, Path]]:
    if not evidence_dir.is_dir():
        fail(f"missing evidence directory: {evidence_dir.relative_to(ROOT)}")
    found: list[tuple[int, Path]] = []
    for path in sorted(evidence_dir.glob(f"{target}__release-default__run*.json")):
        match = RUN_FILE.match(path.name)
        if match is None or match.group("target") != target:
            continue
        found.append((int(match.group("run")), path))
    return found


def validate_performance_bundle(
    matrix: dict[str, Any],
    suite: dict[str, Any],
    candidate_revision: str,
    performance_gate: Any,
) -> list[dict[str, Any]]:
    evidence_dir, profiles, independent_runs = validate_frozen_protocol(matrix, suite)
    reports: list[dict[str, Any]] = []
    evidence_revisions: set[str] = set()
    execution_ids: set[str] = set()
    harness_sha256 = matrix["performance"]["harness_sha256"]

    for profile_id in profiles:
        target, profile = profile_id.split("/", 1)
        files = evidence_files_for_target(evidence_dir, target)
        by_run = {run: path for run, path in files}
        if len(by_run) != len(files):
            fail(f"{target}: duplicate run indexes in evidence filenames")
        missing = [
            index
            for index in range(1, independent_runs + 1)
            if index not in by_run
        ]
        if missing:
            fail(f"{target}: missing independent run files for indexes {missing}")
        selected = [(index, by_run[index]) for index in range(1, independent_runs + 1)]

        for run_index, path in selected:
            data = load_json(path)
            cell = f"{target}/run{run_index}"
            if data.get("target") != target or data.get("profile") != profile:
                fail(f"{cell}: target/profile provenance mismatch")
            recorded_run = data.get("run_index")
            if recorded_run is not None and int(recorded_run) != run_index:
                fail(f"{cell}: run_index field does not match filename")
            if data.get("synthetic") is not False:
                fail(f"{cell}: evidence must explicitly set synthetic=false")
            if data.get("clean_worktree") is not True:
                fail(f"{cell}: evidence was not measured from a clean worktree")
            if data.get("git_revision") != candidate_revision:
                fail(f"{cell}: evidence is stale or from another candidate")

            evidence_revision = data.get("evidence_revision")
            execution_id = data.get("execution_id")
            if not isinstance(evidence_revision, str) or not evidence_revision:
                fail(f"{cell}: missing evidence_revision")
            if evidence_revision in evidence_revisions:
                fail(f"{cell}: duplicate evidence_revision")
            evidence_revisions.add(evidence_revision)
            if not isinstance(execution_id, str) or len(execution_id) < 16:
                fail(f"{cell}: missing independent execution_id")
            if execution_id in execution_ids:
                fail(f"{cell}: duplicate execution_id across independent runs")
            execution_ids.add(execution_id)

            artifacts = data.get("artifacts")
            if not isinstance(artifacts, dict):
                fail(f"{cell}: missing artifact hashes")
            require_digest(
                artifacts.get("harness_sha256"), f"{cell}.artifacts.harness_sha256"
            )
            if artifacts["harness_sha256"] != harness_sha256:
                fail(f"{cell}: benchmark harness hash mismatch")
            for name in ("perf_oxiland_sha256", "perf_redland_sha256"):
                require_digest(artifacts.get(name), f"{cell}.artifacts.{name}")

            report = performance_gate.evaluate(data, suite)
            if report.get("passed") is not True:
                failed_cases = [
                    case["id"] for case in report["cases"] if not case["passed"]
                ]
                failed_resources = [
                    item["id"]
                    for item in report["resource_checks"]
                    if not item["passed"]
                ]
                fail(
                    f"{cell}: performance gate failed: "
                    f"{failed_cases + failed_resources}"
                )
            reports.append(report)
    return reports


def evaluate() -> dict[str, Any]:
    matrix = load_json(MATRIX)
    performance = matrix.get("performance")
    if not isinstance(performance, dict):
        fail("0.13 matrix is missing performance configuration")
    suite_path = resolve_repo_path(performance.get("suite"), "performance.suite")
    suite = load_json(suite_path)
    evidence_dir, profiles, independent_runs = validate_frozen_protocol(matrix, suite)

    head = git_head()
    candidate_revision = head
    if profiles:
        first_target = profiles[0].split("/", 1)[0]
        files = evidence_files_for_target(evidence_dir, first_target)
        if files:
            measured = load_json(files[0][1]).get("git_revision")
            if isinstance(measured, str) and measured:
                candidate_revision = measured
    if not git_is_ancestor(candidate_revision, head):
        fail("0.13 performance evidence is not on the ancestry of HEAD")

    performance_gate = load_script(
        "performance_gate_for_013", ROOT / "scripts" / "check-performance-gate.py"
    )
    reports = validate_performance_bundle(
        matrix, suite, candidate_revision, performance_gate
    )
    return {
        "passed": True,
        "candidate_revision": candidate_revision,
        "independent_runs": independent_runs,
        "targets": len(profiles),
        "evidence_files": len(reports),
        "performance_cases": sum(len(report["cases"]) for report in reports),
        "resource_checks": sum(len(report["resource_checks"]) for report in reports),
    }


def main() -> int:
    try:
        report = evaluate()
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"0.13 release gate failed: {error}", file=sys.stderr)
        return 1
    print("0.13 suite-wide faster-than-Redland qualification passes")
    print(
        f"revision={report['candidate_revision']} "
        f"targets={report['targets']} "
        f"independent_runs={report['independent_runs']} "
        f"evidence_files={report['evidence_files']} "
        f"performance_cases={report['performance_cases']} "
        f"resource_checks={report['resource_checks']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
