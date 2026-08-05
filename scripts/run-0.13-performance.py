#!/usr/bin/env python3
"""Collect strict, per-sample paired Oxiland/Redland 0.13 evidence."""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import uuid
from pathlib import Path
from statistics import median
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
SUITE_PATH = ROOT / "compatibility/performance/0.13-suite.json"
HARNESS = ROOT / "compatibility/harness/c_oracle/perf_bench_0_13.c"
BIN_DIR = ROOT / "compatibility/harness/c_oracle/bin"
OUT_DIR = ROOT / "compatibility/qualification/performance/0.13"

SIZE_HINTS = {
    "P-MUT-1K": 1_000.0,
    "P-MUT-10K": 10_000.0,
    "P-SCAN-10K": 10_000.0,
    "P-PARSE-TTL-1K": 1_000.0,
    "P-PARSE-TTL-10K": 10_000.0,
    "P-SER-NQ-10K": 10_000.0,
    "P-ASK-10K": 1.0,
    "P-SELECT-10K": 1_000.0,
    "P-GRAPH-10K": 1_000.0,
    "P-CALL-100K": 100_000.0,
}


def load_script(name: str, path: Path) -> Any:
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise SystemExit(f"unable to load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


legacy = load_script("build_012_performance", ROOT / "scripts/build-0.12-performance-evidence.py")
gate = load_script("performance_gate_013", ROOT / "scripts/check-performance-gate.py")


def binary(name: str) -> Path:
    plain = BIN_DIR / name
    windows = BIN_DIR / f"{name}.exe"
    return windows if windows.is_file() else plain


def resolve_bash() -> list[str]:
    """Return a bash argv that is not WSL's stub on Windows CI."""
    if sys.platform != "win32":
        return ["bash"]
    # GitHub-hosted Windows runners often put a WSL `bash` ahead of Git/MSYS2.
    # Prefer known native shells; fall back to PATH only if needed.
    candidates = [
        Path(os.environ["ProgramFiles"]) / "Git" / "bin" / "bash.exe"
        if "ProgramFiles" in os.environ
        else None,
        Path(r"C:\Program Files\Git\bin\bash.exe"),
        Path(r"C:\Program Files\Git\usr\bin\bash.exe"),
        Path("/ucrt64/bin/bash.exe"),
        Path("/usr/bin/bash.exe"),
        Path(r"C:\msys64\usr\bin\bash.exe"),
        Path(r"D:\a\_temp\msys64\usr\bin\bash.exe"),
    ]
    for candidate in candidates:
        if candidate is not None and candidate.is_file():
            return [str(candidate)]
    which = shutil.which("bash")
    if which:
        return [which]
    raise SystemExit("bash not found for rebuilding 0.13 performance binaries")


def ensure_binaries() -> tuple[Path, Path]:
    oxiland = binary("perf-oxiland-0.13")
    redland = binary("perf-redland-0.13")
    if oxiland.is_file() and redland.is_file():
        return oxiland, redland
    subprocess.check_call(
        [
            *resolve_bash(),
            str(ROOT / "compatibility/harness/c_oracle/build.sh"),
        ],
        cwd=ROOT,
    )
    oxiland = binary("perf-oxiland-0.13")
    redland = binary("perf-redland-0.13")
    if not oxiland.is_file() or not redland.is_file():
        raise SystemExit("0.13 performance binaries were not built")
    return oxiland, redland


def clean_worktree() -> bool:
    """Like the 0.12 collector, but ignore the 0.13 evidence directory."""
    status = subprocess.check_output(
        ["git", "status", "--porcelain", "--untracked-files=all"],
        cwd=ROOT,
        text=True,
    )
    out_prefix = str(OUT_DIR.relative_to(ROOT)).replace("\\", "/")
    for line in status.splitlines():
        path = line[3:].strip().replace("\\", "/")
        if path.startswith(out_prefix + "/") or path == out_prefix:
            continue
        if path:
            return False
    return True


def run_sample(executable: Path, case_id: str, target_ms: float) -> float:
    proc = subprocess.run(
        [
            str(executable),
            "--case",
            case_id,
            "--samples",
            "1",
            "--target-ms",
            str(target_ms),
        ],
        cwd=ROOT,
        env=legacy.runtime_environment(executable),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=3600,
    )
    if proc.returncode != 0:
        raise SystemExit(f"{executable.name} {case_id} failed: {proc.stderr or proc.stdout}")
    payload = None
    for line in reversed(proc.stdout.splitlines()):
        if line.strip().startswith("{"):
            payload = json.loads(line)
            break
    if payload is None or len(payload.get("seconds", [])) != 1:
        raise SystemExit(f"invalid sample from {executable.name} for {case_id}")
    value = float(payload["seconds"][0])
    if value <= 0:
        raise SystemExit(f"non-positive sample from {executable.name} for {case_id}")
    return value


def paired_case_samples(
    oxiland: Path,
    redland: Path,
    case_id: str,
    count: int,
    target_ms: float,
) -> tuple[list[float], list[float]]:
    oxiland_samples: list[float] = []
    redland_samples: list[float] = []
    for index in range(count):
        if index % 2 == 0:
            ox = run_sample(oxiland, case_id, target_ms)
            red = run_sample(redland, case_id, target_ms)
        else:
            red = run_sample(redland, case_id, target_ms)
            ox = run_sample(oxiland, case_id, target_ms)
        oxiland_samples.append(ox)
        redland_samples.append(red)
        if (index + 1) % 10 == 0:
            print(f"  {case_id}: {index + 1}/{count} pairs", flush=True)
    return oxiland_samples, redland_samples


def rss_samples(executable: Path, case_id: str, count: int) -> list[float]:
    return [legacy.measure_rss_mb(executable, case_id) for _ in range(count)]


def default_output_path(target: str, run_index: int | None) -> Path:
    if run_index is None:
        return OUT_DIR / f"{target}__release-default.json"
    return OUT_DIR / f"{target}__release-default__run{run_index}.json"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    parser.add_argument(
        "--run-index",
        type=int,
        metavar="N",
        help="Independent run number (writes {target}__release-default__runN.json)",
    )
    args = parser.parse_args()
    if args.run_index is not None and args.run_index < 1:
        raise SystemExit("--run-index must be >= 1")

    suite = json.loads(SUITE_PATH.read_text(encoding="utf-8"))
    sample_count = int(suite["thresholds"]["minimum_samples"])
    target_ms = float(suite["protocol"]["minimum_sample_duration_ms"])
    oxiland_bin, redland_bin = ensure_binaries()

    cases = []
    for case in suite["cases"]:
        case_id = case["id"]
        print(f"measuring paired {case_id}...", flush=True)
        ox_seconds, red_seconds = paired_case_samples(
            oxiland_bin, redland_bin, case_id, sample_count, target_ms
        )
        if case["kind"] == "throughput":
            size = SIZE_HINTS[case_id]
            oxiland_values = [size / seconds for seconds in ox_seconds]
            redland_values = [size / seconds for seconds in red_seconds]
            unit = "ops/s"
        else:
            oxiland_values = ox_seconds
            redland_values = red_seconds
            unit = "seconds"
        cases.append(
            {
                "id": case_id,
                "kind": case["kind"],
                "required": True,
                "unit": unit,
                "oxiland": oxiland_values,
                "redland": redland_values,
            }
        )

    print("measuring RSS budgets...", flush=True)
    rss_count = int(suite["protocol"]["resource_samples"])
    ox_parse_samples = rss_samples(oxiland_bin, "P-PARSE-TTL-10K", rss_count)
    red_parse_samples = rss_samples(redland_bin, "P-PARSE-TTL-10K", rss_count)
    ox_query_samples = rss_samples(oxiland_bin, "P-SELECT-10K", rss_count)
    red_query_samples = rss_samples(redland_bin, "P-SELECT-10K", rss_count)
    ox_parse_rss = median(ox_parse_samples)
    red_parse_rss = median(red_parse_samples)
    ox_query_rss = median(ox_query_samples)
    red_query_rss = median(red_query_samples)
    budgets = {item["id"]: item for item in suite["resource_budgets"]}
    resources = [
        {
            "id": "R-RSS-PARSE",
            "unit": "ratio",
            "observed": ox_parse_rss / red_parse_rss,
            "maximum": float(budgets["R-RSS-PARSE"]["maximum"]),
            "oxiland_mib": ox_parse_rss,
            "redland_mib": red_parse_rss,
            "oxiland_samples_mib": ox_parse_samples,
            "redland_samples_mib": red_parse_samples,
        },
        {
            "id": "R-RSS-QUERY",
            "unit": "ratio",
            "observed": ox_query_rss / red_query_rss,
            "maximum": float(budgets["R-RSS-QUERY"]["maximum"]),
            "oxiland_mib": ox_query_rss,
            "redland_mib": red_query_rss,
            "oxiland_samples_mib": ox_query_samples,
            "redland_samples_mib": red_query_samples,
        },
    ]

    target = legacy.host_target()
    execution_id = uuid.uuid4().hex
    is_clean = clean_worktree()
    payload = {
        "schema_version": 1,
        "suite_revision": suite["id"],
        "evidence_revision": f"oxiland-0.13-perf-{target}-{execution_id[:12]}",
        "execution_id": execution_id,
        "target": target,
        "profile": "release-default",
        "run_index": args.run_index,
        "oracle": "system librdf + Oxiland librdf-compat strict paired C bench",
        "host": f"{legacy.platform.system()}/{legacy.platform.machine()}",
        "synthetic": False,
        "clean_worktree": is_clean,
        "git_revision": legacy.git_revision(),
        "build": {
            "oxiland": {
                "cargo_profile": "release",
                "cargo_flags": ["--release", "--locked"],
                "debug_assertions": False,
                "artifact_dir": "target/release",
            },
            "redland": {"cflags": "-O3 -DNDEBUG -march=native", "optimization": "-O3"},
            "c_harness": {"wrapper_flags": "-O3 -DNDEBUG -march=native -Wall -Werror"},
        },
        "artifacts": {
            "harness_sha256": legacy.sha256_file(HARNESS),
            "perf_oxiland_sha256": legacy.sha256_file(oxiland_bin),
            "perf_redland_sha256": legacy.sha256_file(redland_bin),
        },
        "cases": cases,
        "resource_checks": resources,
        "notes": "True per-sample AB/BA pairs; each sample calibrated to at least 10 ms.",
    }

    output = args.output or default_output_path(target, args.run_index)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {output}")

    report = gate.evaluate(payload, suite)
    failed = [case["id"] for case in report["cases"] if not case["passed"]]
    failed += [item["id"] for item in report["resource_checks"] if not item["passed"]]
    if failed:
        print(f"strict gate failed: {', '.join(failed)}", file=sys.stderr)
        return 1
    print("strict gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
