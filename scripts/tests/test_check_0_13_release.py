#!/usr/bin/env python3
"""Unit tests for the fail-closed 0.13 suite-wide performance checker."""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def load_checker():
    spec = importlib.util.spec_from_file_location(
        "check_013", ROOT / "scripts" / "check-0.13-release.py"
    )
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class PassingGate:
    @staticmethod
    def evaluate(data, suite):
        return {
            "passed": True,
            "cases": [{"id": "case", "passed": True}],
            "resource_checks": [{"id": "rss", "passed": True}],
        }


class FailingGate:
    @staticmethod
    def evaluate(data, suite):
        return {
            "passed": False,
            "cases": [{"id": "P-CALL-100K", "passed": False}],
            "resource_checks": [{"id": "R-RSS-PARSE", "passed": True}],
        }


class Check013Tests(unittest.TestCase):
    def setUp(self) -> None:
        self.checker = load_checker()
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.suite_path = self.root / "suite.json"
        self.harness_path = self.root / "bench.c"
        self.suite_path.write_text("suite", encoding="utf-8")
        self.harness_path.write_text("bench", encoding="utf-8")
        self.suite = {
            "schema_version": 1,
            "milestone": "0.13",
            "status": "frozen",
            "thresholds": {"independent_host_runs": 3},
            "cases": [{"id": "case", "required": True}],
            "resource_budgets": [{"id": "rss", "maximum": 1.25}],
        }
        self.profile = "aarch64-apple-darwin/release-default"
        self.matrix = {
            "schema_version": 1,
            "milestone": "0.13",
            "status": "frozen",
            "targets": ["aarch64-apple-darwin"],
            "performance": {
                "suite": "suite.json",
                "suite_sha256": self.checker.sha256_file(self.suite_path),
                "harness": "bench.c",
                "harness_sha256": self.checker.sha256_file(self.harness_path),
                "evidence_dir": "evidence",
                "independent_runs": 3,
                "profiles": [self.profile],
            },
        }
        self.original_root = self.checker.ROOT
        self.checker.ROOT = self.root
        self.addCleanup(setattr, self.checker, "ROOT", self.original_root)

    def evidence(self, run_index: int, execution_suffix: str) -> dict:
        digest = "a" * 64
        return {
            "target": "aarch64-apple-darwin",
            "profile": "release-default",
            "run_index": run_index,
            "synthetic": False,
            "clean_worktree": True,
            "git_revision": "candidate",
            "evidence_revision": f"native-macos-run{run_index}-{execution_suffix}",
            "execution_id": f"{execution_suffix}{run_index:02d}" + ("0" * 14),
            "artifacts": {
                "harness_sha256": self.matrix["performance"]["harness_sha256"],
                "perf_oxiland_sha256": digest,
                "perf_redland_sha256": "b" * 64,
            },
        }

    def write_runs(self, count: int = 3) -> None:
        evidence_dir = self.root / "evidence"
        evidence_dir.mkdir(exist_ok=True)
        for run in range(1, count + 1):
            path = evidence_dir / f"aarch64-apple-darwin__release-default__run{run}.json"
            path.write_text(
                json.dumps(self.evidence(run, "abcdef0123456789")), encoding="utf-8"
            )

    def test_draft_protocol_is_release_blocking(self) -> None:
        self.matrix["status"] = "draft"
        with self.assertRaisesRegex(ValueError, "matrix is not frozen"):
            self.checker.validate_frozen_protocol(self.matrix, self.suite)

    def test_missing_run_is_release_blocking(self) -> None:
        self.write_runs(2)
        with self.assertRaisesRegex(ValueError, "missing independent run"):
            self.checker.validate_performance_bundle(
                self.matrix, self.suite, "candidate", PassingGate()
            )

    def test_duplicate_execution_id_is_rejected(self) -> None:
        evidence_dir = self.root / "evidence"
        evidence_dir.mkdir()
        for run in range(1, 4):
            payload = self.evidence(run, "sameexecutionid00")
            payload["execution_id"] = "sameexecutionid000"
            path = evidence_dir / f"aarch64-apple-darwin__release-default__run{run}.json"
            path.write_text(json.dumps(payload), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "duplicate execution_id"):
            self.checker.validate_performance_bundle(
                self.matrix, self.suite, "candidate", PassingGate()
            )

    def test_failed_case_is_release_blocking(self) -> None:
        self.write_runs(3)
        with self.assertRaisesRegex(ValueError, "performance gate failed"):
            self.checker.validate_performance_bundle(
                self.matrix, self.suite, "candidate", FailingGate()
            )

    def test_three_independent_runs_pass(self) -> None:
        self.write_runs(3)
        reports = self.checker.validate_performance_bundle(
            self.matrix, self.suite, "candidate", PassingGate()
        )
        self.assertEqual(len(reports), 3)


if __name__ == "__main__":
    unittest.main()
