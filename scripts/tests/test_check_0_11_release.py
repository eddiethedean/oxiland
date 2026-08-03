#!/usr/bin/env python3
"""Unit tests for the fail-closed 0.11 release checker."""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def load_checker():
    spec = importlib.util.spec_from_file_location(
        "check_011", ROOT / "scripts" / "check-0.11-release.py"
    )
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class Check011Tests(unittest.TestCase):
    def test_rejects_synthetic_raw_result(self) -> None:
        checker = load_checker()
        with tempfile.TemporaryDirectory() as tmp:
            raw = Path(tmp)
            bad = {
                "schema_version": 1,
                "milestone": "0.11",
                "fixture_id": "x",
                "profile_id": "aarch64-apple-darwin/release-default",
                "target": "aarch64-apple-darwin",
                "synthetic": True,
                "differential_passed": True,
                "comparison": {"passed": True, "mismatches": []},
                "skips": [],
                "quarantined": [],
                "deviations": [],
                "clean_worktree": True,
                "execution_id": "a" * 32,
                "git_revision": "abc",
            }
            (raw / "one.json").write_text(json.dumps(bad), encoding="utf-8")
            with self.assertRaises(ValueError) as ctx:
                checker.validate_raw(
                    [{**bad, "_path": "one.json"}],
                    {
                        "required_profile_ids": [
                            "aarch64-apple-darwin/release-default"
                        ]
                    },
                    "abc",
                )
            self.assertIn("synthetic", str(ctx.exception))

    def test_rejects_profile_fan_out(self) -> None:
        checker = load_checker()
        shared = "shared-execution-id-0123456789abcdef"
        results = []
        for target, profile in [
            ("aarch64-apple-darwin", "aarch64-apple-darwin/release-default"),
            ("x86_64-unknown-linux-gnu", "x86_64-unknown-linux-gnu/release-default"),
        ]:
            results.append(
                {
                    "schema_version": 1,
                    "milestone": "0.11",
                    "fixture_id": "world-lifecycle",
                    "profile_id": profile,
                    "target": target,
                    "synthetic": False,
                    "differential_passed": True,
                    "comparison": {"passed": True, "mismatches": []},
                    "skips": [],
                    "quarantined": [],
                    "deviations": [],
                    "clean_worktree": True,
                    "execution_id": shared,
                    "git_revision": "abc",
                    "obligation_ids": [],
                    "oxiland": {"engine": "oxiland-c", "ok": True},
                    "redland": {"engine": "redland-c", "ok": True},
                    "artifacts": {
                        "oxiland_library": "/tmp/liboxiland.dylib",
                        "redland_library": "/tmp/librdf.dylib",
                    },
                    "_path": f"{target}.json",
                }
            )
        with tempfile.TemporaryDirectory() as tmp:
            raw = Path(tmp)
            # Pretend indexes exist by patching RAW? validate_raw checks RAW path on disk.
            # Call lower-level fan-out detection by invoking validate_raw with monkeypatch.
            original = checker.RAW
            checker.RAW = raw
            try:
                for result in results:
                    name = result["profile_id"].replace("/", "__") + "__index.json"
                    (raw / name).write_text(
                        json.dumps(
                            {
                                "schema_version": 1,
                                "milestone": "0.11",
                                "profile_id": result["profile_id"],
                                "failed": 0,
                                "synthetic": False,
                                "git_revision": "abc",
                            }
                        ),
                        encoding="utf-8",
                    )
                with self.assertRaises(ValueError) as ctx:
                    checker.validate_raw(
                        results,
                        {
                            "required_profile_ids": [
                                "aarch64-apple-darwin/release-default",
                                "x86_64-unknown-linux-gnu/release-default",
                            ]
                        },
                        "abc",
                    )
                self.assertIn("fan-out", str(ctx.exception))
            finally:
                checker.RAW = original

    def test_rejects_planned_fuzz_duration(self) -> None:
        checker = load_checker()
        soak = {
            "milestone": "0.11",
            "completed": True,
            "abi_resets": 0,
            "release_blockers": [],
            "git_revision": "abc",
        }
        fuzz = {
            "milestone": "0.11",
            "findings": [],
            "targets": [
                {
                    "name": "rdf_parser",
                    "duration_seconds": 3600,
                    "duration_status": "planned",
                    "result": "pass",
                    "findings": [],
                },
                {
                    "name": "c_lifecycle",
                    "duration_seconds": 3600,
                    "duration_status": "executed",
                    "result": "pass",
                    "findings": [],
                },
            ],
        }
        with self.assertRaises(ValueError) as ctx:
            checker.validate_soak_fuzz(soak, fuzz, "abc")
        self.assertIn("duration_status", str(ctx.exception))

    def test_accepts_executed_fuzz_duration(self) -> None:
        checker = load_checker()
        soak = {
            "milestone": "0.11",
            "completed": True,
            "abi_resets": 0,
            "release_blockers": [],
            "git_revision": "abc",
        }
        fuzz = {
            "milestone": "0.11",
            "findings": [],
            "targets": [
                {
                    "name": "rdf_parser",
                    "duration_seconds": 3600,
                    "duration_status": "executed",
                    "result": "pass",
                    "findings": [],
                },
                {
                    "name": "c_lifecycle",
                    "duration_seconds": 3600,
                    "duration_status": "executed",
                    "result": "pass",
                    "findings": [],
                },
            ],
        }
        checker.validate_soak_fuzz(soak, fuzz, "abc")


if __name__ == "__main__":
    unittest.main()
