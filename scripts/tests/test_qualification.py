from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


def load_script(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / filename)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


performance = load_script("performance_gate", "check-performance-gate.py")
parity = load_script("parity_gate", "check-0.10-parity.py")
inventory = load_script("inventory_gate", "check-inventory.py")
release = load_script("release_gate", "check-0.10-release.py")


class PerformanceGateTests(unittest.TestCase):
    def data(self) -> dict:
        return {
            "schema_version": 1,
            "suite_revision": "fixture-v1",
            "evidence_revision": "deadbeef",
            "target": "x86_64-unknown-linux-gnu",
            "profile": "release-default",
            "oracle": "redland-1.0.17",
            "host": "test-host",
            "cases": [
                {
                    "id": "throughput",
                    "kind": "throughput",
                    "unit": "items/s",
                    "required": True,
                    "oxiland": [120 + index % 5 for index in range(30)],
                    "redland": [100 + index % 5 for index in range(30)],
                },
                {
                    "id": "latency",
                    "kind": "latency",
                    "unit": "ns/op",
                    "required": True,
                    "oxiland": [70 + index % 5 for index in range(30)],
                    "redland": [100 + index % 5 for index in range(30)],
                },
            ],
            "resource_checks": [
                {"id": "rss", "unit": "MiB", "observed": 10, "maximum": 20}
            ],
        }

    def suite(self, data: dict) -> dict:
        return {
            "schema_version": 1,
            "id": data["suite_revision"],
            "thresholds": {
                "throughput_oxiland_over_redland_min": 1.05,
                "latency_oxiland_over_redland_max": 0.95,
                "bootstrap_rounds": 10000,
                "minimum_samples": 30,
            },
            "cases": [{"id": item["id"], "required": True} for item in data["cases"]],
            "resource_budgets": [{"id": "rss", "maximum": 20}],
        }

    def test_passing_matrix_requires_each_case_and_resource_to_win(self) -> None:
        data = self.data()
        report = performance.evaluate(data, self.suite(data))
        self.assertTrue(report["passed"])
        self.assertTrue(all(case["passed"] for case in report["cases"]))

    def test_one_loss_cannot_be_averaged_away(self) -> None:
        data = self.data()
        data["cases"][1]["oxiland"] = [110 + index % 5 for index in range(30)]
        report = performance.evaluate(data, self.suite(data))
        self.assertFalse(report["passed"])
        self.assertFalse(report["cases"][1]["passed"])

    def test_required_case_cannot_be_deleted(self) -> None:
        data = self.data()
        suite = self.suite(data)
        data["cases"].pop()
        with self.assertRaisesRegex(ValueError, "exactly the frozen required cases"):
            performance.evaluate(data, suite)


class ParityGateTests(unittest.TestCase):
    def write_inputs(self, directory: Path, c_state: str = "verified") -> tuple[Path, Path]:
        symbols = ["librdf_new_world", "librdf_free_world"]
        entries = [
            {
                "symbol": symbols[0],
                "state": "verified",
                "c_state": c_state,
                "deviations": [],
            },
            {
                "symbol": symbols[1],
                "state": "not-applicable",
                "safe_n_a_kind": "ownership-mechanic",
                "c_state": "verified",
                "deviations": [],
            },
        ]
        inventory_path = directory / "inventory.json"
        inventory_path.write_text(
            json.dumps(
                {
                    "milestone": "0.10",
                    "oxiland_version": "0.10.0",
                    "redland_api": "1.0.17",
                    "entries": entries,
                }
            )
        )
        evidence_path = directory / "evidence.json"
        evidence_path.write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "expected_profiles": ["linux-default"],
                    "profiles": [
                        {
                            "id": "linux-default",
                            "target": "x86_64-unknown-linux-gnu",
                            "build_profile": "release-default",
                            "oracle": "redland-1.0.17",
                            "evidence_revision": "deadbeef",
                            "verified_symbols": symbols,
                            "differential_passed": True,
                            "skips": [],
                            "mismatches": [],
                            "quarantined": [],
                            "deviations": [],
                        }
                    ],
                }
            )
        )
        return inventory_path, evidence_path

    def test_exact_complete_matrix_passes(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            inventory_path, evidence_path = self.write_inputs(Path(raw))
            self.assertTrue(parity.evaluate(inventory_path, evidence_path)["passed"])

    def test_c_exclusion_is_release_blocking(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            inventory_path, evidence_path = self.write_inputs(Path(raw), "excluded")
            self.assertFalse(parity.evaluate(inventory_path, evidence_path)["passed"])

    def test_inventory_version_must_match_release(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            inventory_path, evidence_path = self.write_inputs(Path(raw))
            data = json.loads(inventory_path.read_text())
            data["oxiland_version"] = "0.9.0"
            inventory_path.write_text(json.dumps(data))
            with self.assertRaisesRegex(ValueError, "version must be 0.10.0"):
                parity.evaluate(inventory_path, evidence_path)

    def test_numeric_milestone_order(self) -> None:
        self.assertGreater(inventory.milestone_key("0.10"), inventory.milestone_key("0.9"))


class ReleaseScaffoldTests(unittest.TestCase):
    def fuzz_record(self) -> dict:
        return {
            "schema_version": 1,
            "milestone": "0.10",
            "git_revision": "deadbeef",
            "findings": [],
            "targets": [
                {"name": name, "smoke_result": "pass", "findings": []}
                for name in sorted(release.FUZZ_TARGETS)
            ],
        }

    def test_fuzz_smokes_cover_frozen_targets(self) -> None:
        release.validate_fuzz(self.fuzz_record())

    def test_fuzz_target_finding_is_release_blocking(self) -> None:
        record = self.fuzz_record()
        record["targets"][0]["findings"] = ["crash"]
        with self.assertRaisesRegex(ValueError, "retains findings"):
            release.validate_fuzz(record)

    def test_fuzz_target_cannot_be_deleted(self) -> None:
        record = self.fuzz_record()
        record["targets"].pop()
        with self.assertRaisesRegex(ValueError, "frozen target set"):
            release.validate_fuzz(record)


if __name__ == "__main__":
    unittest.main()
