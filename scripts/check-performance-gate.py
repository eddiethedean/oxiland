#!/usr/bin/env python3
"""Evaluate the frozen Oxiland 0.10 faster-than-Redland gate.

The input contains raw paired-host samples. This program intentionally does
not run workloads: benchmark drivers may be platform-specific, while the
qualification arithmetic and failure policy must be identical everywhere.
"""

from __future__ import annotations

import argparse
import json
import math
import random
import statistics
import sys
from pathlib import Path
from typing import Any


BOOTSTRAP_ROUNDS = 10_000
MINIMUM_SAMPLES = 30
THROUGHPUT_THRESHOLD = 1.05
LATENCY_THRESHOLD = 0.95
ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SUITE = ROOT / "compatibility" / "performance" / "0.10-suite.json"


def fail(message: str) -> None:
    raise ValueError(message)


def positive_samples(case_id: str, owner: str, value: Any) -> list[float]:
    if not isinstance(value, list) or len(value) < MINIMUM_SAMPLES:
        fail(f"{case_id}: {owner} requires at least {MINIMUM_SAMPLES} raw samples")
    samples = [float(item) for item in value]
    if any(not math.isfinite(item) or item <= 0 for item in samples):
        fail(f"{case_id}: {owner} samples must be finite and positive")
    return samples


def ratio(kind: str, oxiland: list[float], redland: list[float]) -> float:
    oxiland_median = statistics.median(oxiland)
    redland_median = statistics.median(redland)
    # The normative contract defines Oxiland/Redland for both metrics.
    return oxiland_median / redland_median


def bootstrap_interval(
    kind: str, oxiland: list[float], redland: list[float], seed: int
) -> tuple[float, float]:
    rng = random.Random(seed)
    values: list[float] = []
    for _ in range(BOOTSTRAP_ROUNDS):
        oxiland_resample = [rng.choice(oxiland) for _ in oxiland]
        redland_resample = [rng.choice(redland) for _ in redland]
        values.append(ratio(kind, oxiland_resample, redland_resample))
    values.sort()
    lower = values[math.floor(0.025 * (len(values) - 1))]
    upper = values[math.ceil(0.975 * (len(values) - 1))]
    return lower, upper


def evaluate_case(case: dict[str, Any], index: int) -> dict[str, Any]:
    case_id = case.get("id")
    if not isinstance(case_id, str) or not case_id:
        fail(f"case {index}: missing id")
    kind = case.get("kind")
    if kind not in {"throughput", "latency"}:
        fail(f"{case_id}: kind must be throughput or latency")
    if case.get("required") is not True:
        fail(f"{case_id}: all frozen cases must be required")

    oxiland = positive_samples(case_id, "oxiland", case.get("oxiland"))
    redland = positive_samples(case_id, "redland", case.get("redland"))
    observed = ratio(kind, oxiland, redland)
    lower, upper = bootstrap_interval(kind, oxiland, redland, index)
    if kind == "throughput":
        passed = observed >= THROUGHPUT_THRESHOLD and lower > 1.0
        threshold = THROUGHPUT_THRESHOLD
    else:
        passed = observed <= LATENCY_THRESHOLD and upper < 1.0
        threshold = LATENCY_THRESHOLD

    return {
        "id": case_id,
        "kind": kind,
        "unit": case.get("unit"),
        "oxiland_median": statistics.median(oxiland),
        "redland_median": statistics.median(redland),
        "ratio": observed,
        "confidence_interval_95": [lower, upper],
        "threshold": threshold,
        "sample_count": {"oxiland": len(oxiland), "redland": len(redland)},
        "passed": passed,
    }


def evaluate(data: dict[str, Any], suite: dict[str, Any] | None = None) -> dict[str, Any]:
    if data.get("schema_version") != 1:
        fail("unsupported schema_version")
    required_text = (
        "suite_revision",
        "evidence_revision",
        "target",
        "profile",
        "oracle",
        "host",
    )
    for key in required_text:
        if not isinstance(data.get(key), str) or not data[key].strip():
            fail(f"missing non-empty {key}")
    cases = data.get("cases")
    if not isinstance(cases, list) or not cases:
        fail("cases must be a non-empty list")
    results = [evaluate_case(case, index) for index, case in enumerate(cases)]
    ids = [result["id"] for result in results]
    if len(ids) != len(set(ids)):
        fail("case ids must be unique")

    resource_checks = data.get("resource_checks")
    if not isinstance(resource_checks, list) or not resource_checks:
        fail("resource_checks must contain the frozen memory/disk budgets")
    resources_pass = True
    normalized_resources: list[dict[str, Any]] = []
    for resource in resource_checks:
        if not isinstance(resource.get("id"), str):
            fail("resource check missing id")
        observed = float(resource.get("observed"))
        maximum = float(resource.get("maximum"))
        passed = math.isfinite(observed) and math.isfinite(maximum) and observed <= maximum
        resources_pass &= passed
        normalized_resources.append({**resource, "observed": observed, "maximum": maximum, "passed": passed})

    if suite is not None:
        if suite.get("schema_version") != 1 or data["suite_revision"] != suite.get("id"):
            fail("raw samples do not match the frozen suite revision")
        expected_cases = {item["id"] for item in suite.get("cases", []) if item.get("required") is True}
        if set(ids) != expected_cases:
            fail("raw samples do not contain exactly the frozen required cases")
        expected_resources = {item["id"] for item in suite.get("resource_budgets", [])}
        actual_resources = {item["id"] for item in normalized_resources}
        if actual_resources != expected_resources:
            fail("raw samples do not contain exactly the frozen resource budgets")
        frozen_maximums = {
            item["id"]: float(item["maximum"]) for item in suite.get("resource_budgets", [])
        }
        if any(
            item["maximum"] != frozen_maximums[item["id"]] for item in normalized_resources
        ):
            fail("raw samples changed a frozen resource budget")
        thresholds = suite.get("thresholds", {})
        if (
            thresholds.get("throughput_oxiland_over_redland_min") != THROUGHPUT_THRESHOLD
            or thresholds.get("latency_oxiland_over_redland_max") != LATENCY_THRESHOLD
            or thresholds.get("bootstrap_rounds") != BOOTSTRAP_ROUNDS
            or thresholds.get("minimum_samples") != MINIMUM_SAMPLES
        ):
            fail("frozen suite thresholds disagree with the qualification implementation")

    return {
        "schema_version": 1,
        "suite_revision": data["suite_revision"],
        "evidence_revision": data["evidence_revision"],
        "target": data["target"],
        "profile": data["profile"],
        "oracle": data["oracle"],
        "host": data["host"],
        "bootstrap_rounds": BOOTSTRAP_ROUNDS,
        "cases": results,
        "resource_checks": normalized_resources,
        "passed": all(result["passed"] for result in results) and resources_pass,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path, help="raw benchmark JSON")
    parser.add_argument("--suite", type=Path, default=DEFAULT_SUITE, help="frozen suite JSON")
    parser.add_argument("--output", type=Path, help="write normalized report JSON")
    args = parser.parse_args()
    try:
        data = json.loads(args.input.read_text(encoding="utf-8"))
        suite = json.loads(args.suite.read_text(encoding="utf-8"))
        report = evaluate(data, suite)
    except (OSError, json.JSONDecodeError, TypeError, ValueError) as error:
        print(f"performance gate error: {error}", file=sys.stderr)
        return 2

    encoded = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(encoded, encoding="utf-8")
    else:
        print(encoded, end="")
    if not report["passed"]:
        failed = [case["id"] for case in report["cases"] if not case["passed"]]
        failed += [item["id"] for item in report["resource_checks"] if not item["passed"]]
        print(f"performance gate failed: {', '.join(failed)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
