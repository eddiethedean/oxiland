#!/usr/bin/env python3
"""Unit tests for the fail-closed 0.12 release checker."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def load_checker():
    spec = importlib.util.spec_from_file_location(
        "check_012", ROOT / "scripts" / "check-0.12-release.py"
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


class Check012Tests(unittest.TestCase):
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
            "milestone": "0.12",
            "status": "frozen",
            "cases": [{"id": "case", "required": True}],
            "resource_budgets": [{"id": "rss", "maximum": 1.25}],
        }
        self.profile = "aarch64-apple-darwin/release-default"
        self.matrix = {
            "schema_version": 1,
            "milestone": "0.12",
            "status": "frozen",
            "targets": ["aarch64-apple-darwin"],
            "performance": {
                "suite": "suite.json",
                "suite_sha256": self.checker.sha256_file(self.suite_path),
                "harness": "bench.c",
                "harness_sha256": self.checker.sha256_file(self.harness_path),
                "evidence_dir": "evidence",
                "profiles": [self.profile],
            },
        }
        self.original_root = self.checker.ROOT
        self.checker.ROOT = self.root
        self.addCleanup(setattr, self.checker, "ROOT", self.original_root)

    def evidence(self) -> dict:
        digest = "a" * 64
        return {
            "target": "aarch64-apple-darwin",
            "profile": "release-default",
            "synthetic": False,
            "clean_worktree": True,
            "git_revision": "candidate",
            "evidence_revision": "native-macos-v1",
            "execution_id": "0123456789abcdef",
            "artifacts": {
                "oxiland_library_sha256": digest,
                "redland_library_sha256": "b" * 64,
                "harness_sha256": self.matrix["performance"]["harness_sha256"],
            },
        }

    def test_draft_protocol_is_release_blocking(self) -> None:
        self.matrix["status"] = "draft"
        with self.assertRaisesRegex(ValueError, "matrix is not frozen"):
            self.checker.validate_frozen_protocol(self.matrix, self.suite)

    def test_missing_resource_budgets_are_release_blocking(self) -> None:
        self.suite["resource_budgets"] = []
        with self.assertRaisesRegex(ValueError, "resource budgets are not frozen"):
            self.checker.validate_frozen_protocol(self.matrix, self.suite)

    def test_synthetic_performance_evidence_is_rejected(self) -> None:
        evidence_dir = self.root / "evidence"
        evidence_dir.mkdir()
        evidence = self.evidence()
        evidence["synthetic"] = True
        path = evidence_dir / "aarch64-apple-darwin__release-default.json"
        import json

        path.write_text(json.dumps(evidence), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "synthetic=false"):
            self.checker.validate_performance_bundle(
                self.matrix, self.suite, "candidate", PassingGate()
            )

    def test_stale_performance_evidence_is_rejected(self) -> None:
        evidence_dir = self.root / "evidence"
        evidence_dir.mkdir()
        evidence = self.evidence()
        evidence["git_revision"] = "old"
        path = evidence_dir / "aarch64-apple-darwin__release-default.json"
        import json

        path.write_text(json.dumps(evidence), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "stale"):
            self.checker.validate_performance_bundle(
                self.matrix, self.suite, "candidate", PassingGate()
            )

    def test_native_candidate_bound_bundle_passes(self) -> None:
        evidence_dir = self.root / "evidence"
        evidence_dir.mkdir()
        path = evidence_dir / "aarch64-apple-darwin__release-default.json"
        import json

        path.write_text(json.dumps(self.evidence()), encoding="utf-8")
        reports = self.checker.validate_performance_bundle(
            self.matrix, self.suite, "candidate", PassingGate()
        )
        self.assertEqual(len(reports), 1)


if __name__ == "__main__":
    unittest.main()
