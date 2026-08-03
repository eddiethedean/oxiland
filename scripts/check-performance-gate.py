#!/usr/bin/env python3
"""Evaluate the frozen Oxiland-versus-Redland performance statistical rules.

The input contains raw paired-host samples. This program intentionally does
not run workloads: benchmark drivers may be platform-specific, while the
qualification arithmetic and failure policy must be identical everywhere.
When a suite sets ``protocol.require_production_compile``, samples must also
prove Cargo ``--release`` / production compile provenance (Rust speed depends
on the compile profile).
Input provenance is enforced by the candidate-bound qualification layer, not
by this arithmetic validator alone.
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
MINIMUM_SAMPLES = 30  # absolute floor; suites may require more via thresholds.minimum_samples
ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SUITE = ROOT / "compatibility" / "performance" / "0.10-suite.json"


def suite_thresholds(suite: dict[str, Any] | None) -> tuple[float, float, float, float]:
    """Return throughput/latency medians and CI bounds from the suite.

    Suites that omit CI bound fields keep the historical faster-than-Redland
    rule (throughput CI lower > 1.0, latency CI upper < 1.0).
    """
    thresholds = suite.get("thresholds", {}) if isinstance(suite, dict) else {}
    throughput = float(thresholds.get("throughput_oxiland_over_redland_min", 1.05))
    latency = float(thresholds.get("latency_oxiland_over_redland_max", 0.95))
    ci_lower = float(thresholds.get("throughput_ci_lower_min", 1.0))
    ci_upper = float(thresholds.get("latency_ci_upper_max", 1.0))
    return throughput, latency, ci_lower, ci_upper


def fail(message: str) -> None:
    raise ValueError(message)


def positive_samples(case_id: str, owner: str, value: Any, minimum: int) -> list[float]:
    if not isinstance(value, list) or len(value) < minimum:
        fail(f"{case_id}: {owner} requires at least {minimum} raw samples")
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


def _nonempty_str(value: Any) -> str | None:
    if isinstance(value, str) and value.strip():
        return value.strip()
    return None


def validate_production_compile(data: dict[str, Any], suite: dict[str, Any] | None) -> None:
    """Reject debug/dev Rust builds when the suite requires production compile.

    Rust throughput is dominated by Cargo profile. Qualification evidence must
    use ``cargo build --release`` (profile ``release``), not ``target/debug``.
    """
    protocol = {}
    if suite is not None and isinstance(suite.get("protocol"), dict):
        protocol = suite["protocol"]
    require = protocol.get("require_production_compile") is True
    build = data.get("build")
    if not require and build is None:
        return
    if require and not isinstance(build, dict):
        fail("production-compile suites require a build provenance object")

    assert isinstance(build, dict)
    profile_name = _nonempty_str(data.get("profile")) or ""
    if "debug" in profile_name.lower() or profile_name.lower() in {"dev", "test"}:
        fail(f"profile {profile_name!r} is not a production/release performance profile")

    oxiland = build.get("oxiland")
    if not isinstance(oxiland, dict):
        fail("build.oxiland provenance is required for production-compile evidence")
    cargo_profile = _nonempty_str(oxiland.get("cargo_profile"))
    if cargo_profile != "release":
        fail("build.oxiland.cargo_profile must be 'release' (production compile)")
    if require and oxiland.get("debug_assertions") is not False:
        fail("build.oxiland.debug_assertions must explicitly be false")
    artifact_dir = _nonempty_str(oxiland.get("artifact_dir"))
    if require and artifact_dir is None:
        fail("build.oxiland.artifact_dir is required for production compile")
    if artifact_dir is not None and "debug" in artifact_dir.replace("\\", "/").lower().split("/"):
        fail("build.oxiland.artifact_dir must not point at a debug build")
    flags = oxiland.get("cargo_flags")
    if require and flags is None:
        fail("build.oxiland.cargo_flags are required for production compile")
    if flags is not None:
        if not isinstance(flags, list) or not all(isinstance(item, str) for item in flags):
            fail("build.oxiland.cargo_flags must be a list of strings")
        normalized = {item.strip() for item in flags}
        cargo_protocol = protocol.get("cargo")
        required_flags = ["--release"]
        if isinstance(cargo_protocol, dict) and isinstance(
            cargo_protocol.get("required_flags"), list
        ):
            required_flags = cargo_protocol["required_flags"]
        missing_flags = [flag for flag in required_flags if flag not in normalized]
        if missing_flags:
            fail(
                "build.oxiland.cargo_flags missing required production flags: "
                f"{missing_flags}"
            )

    redland = build.get("redland")
    if require:
        if not isinstance(redland, dict):
            fail("build.redland provenance is required for production-compile evidence")
        opt = _nonempty_str(redland.get("optimization")) or _nonempty_str(redland.get("cflags"))
        if opt is None:
            fail("build.redland must record optimization or cflags")
        if any(token in opt.lower() for token in ("-o0", "opt-level=0", "opt_level=0")):
            fail("build.redland optimization must not be -O0 / opt-level=0")


def evaluate_case(
    case: dict[str, Any],
    index: int,
    throughput_threshold: float,
    latency_threshold: float,
    throughput_ci_lower_min: float,
    latency_ci_upper_max: float,
    minimum_samples: int,
) -> dict[str, Any]:
    case_id = case.get("id")
    if not isinstance(case_id, str) or not case_id:
        fail(f"case {index}: missing id")
    kind = case.get("kind")
    if kind not in {"throughput", "latency"}:
        fail(f"{case_id}: kind must be throughput or latency")
    if case.get("required") is not True:
        fail(f"{case_id}: all frozen cases must be required")

    oxiland = positive_samples(case_id, "oxiland", case.get("oxiland"), minimum_samples)
    redland = positive_samples(case_id, "redland", case.get("redland"), minimum_samples)
    observed = ratio(kind, oxiland, redland)
    lower, upper = bootstrap_interval(kind, oxiland, redland, index)
    if kind == "throughput":
        passed = observed >= throughput_threshold and lower > throughput_ci_lower_min
        threshold = throughput_threshold
    else:
        passed = observed <= latency_threshold and upper < latency_ci_upper_max
        threshold = latency_threshold

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
    validate_production_compile(data, suite)
    (
        throughput_threshold,
        latency_threshold,
        throughput_ci_lower_min,
        latency_ci_upper_max,
    ) = suite_thresholds(suite)
    minimum_samples = MINIMUM_SAMPLES
    if isinstance(suite, dict):
        thresholds = suite.get("thresholds", {})
        if isinstance(thresholds, dict) and thresholds.get("minimum_samples") is not None:
            minimum_samples = max(MINIMUM_SAMPLES, int(thresholds["minimum_samples"]))
    cases = data.get("cases")
    if not isinstance(cases, list) or not cases:
        fail("cases must be a non-empty list")
    results = [
        evaluate_case(
            case,
            index,
            throughput_threshold,
            latency_threshold,
            throughput_ci_lower_min,
            latency_ci_upper_max,
            minimum_samples,
        )
        for index, case in enumerate(cases)
    ]
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
        expected_case_items = {
            item["id"]: item
            for item in suite.get("cases", [])
            if item.get("required") is True
        }
        expected_cases = set(expected_case_items)
        if set(ids) != expected_cases:
            fail("raw samples do not contain exactly the frozen required cases")
        for case in cases:
            frozen = expected_case_items[case["id"]]
            if frozen.get("kind") is not None and case.get("kind") != frozen["kind"]:
                fail(f"{case['id']}: metric kind differs from the frozen suite")
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
        frozen_resources = {
            item["id"]: item for item in suite.get("resource_budgets", [])
        }
        for item in normalized_resources:
            frozen = frozen_resources[item["id"]]
            if frozen.get("unit") is not None and item.get("unit") != frozen["unit"]:
                fail(f"{item['id']}: resource unit differs from the frozen suite")
        thresholds = suite.get("thresholds", {})
        if thresholds.get("bootstrap_rounds") != BOOTSTRAP_ROUNDS:
            fail("frozen suite thresholds disagree with the qualification implementation")
        if int(thresholds.get("minimum_samples", MINIMUM_SAMPLES)) < MINIMUM_SAMPLES:
            fail("frozen suite minimum_samples is below the implementation floor")
        if (
            float(thresholds.get("throughput_oxiland_over_redland_min", 1.05))
            != throughput_threshold
            or float(thresholds.get("latency_oxiland_over_redland_max", 0.95))
            != latency_threshold
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
