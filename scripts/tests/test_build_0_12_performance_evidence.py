#!/usr/bin/env python3
"""Unit tests for the 0.12 performance evidence builder schema."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def load_builder():
    spec = importlib.util.spec_from_file_location(
        "build_012_perf", ROOT / "scripts" / "build-0.12-performance-evidence.py"
    )
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Build012EvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.builder = load_builder()

    def test_size_hints_cover_frozen_suite_cases(self) -> None:
        suite = (
            ROOT / "compatibility" / "performance" / "0.12-suite.json"
        )
        import json

        data = json.loads(suite.read_text(encoding="utf-8"))
        case_ids = {item["id"] for item in data["cases"]}
        self.assertEqual(case_ids, set(self.builder.SIZE_HINTS))

    def test_output_dir_is_0_12_namespace(self) -> None:
        self.assertTrue(str(self.builder.OUT_DIR).endswith("performance/0.12"))

    def test_suite_path_points_at_0_12_suite(self) -> None:
        self.assertEqual(self.builder.SUITE.name, "0.12-suite.json")


if __name__ == "__main__":
    unittest.main()
