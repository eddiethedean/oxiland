#!/usr/bin/env python3
"""Build 0.11 native performance evidence from independent Redland and Oxiland measurements.

Uses C perf benches linked to system librdf and Oxiland librdf-compat for every
suite case. Does not fabricate paired ratios or rescale times. Resource ratios
are omitted unless measured.
"""

from __future__ import annotations

import json
import platform
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SUITE = ROOT / "compatibility/performance/0.11-suite.json"
OUT_DIR = ROOT / "compatibility/qualification/performance"
ORACLE_BIN = ROOT / "compatibility/harness/c_oracle/bin"
BUILD_SH = ROOT / "compatibility/harness/c_oracle/build.sh"

SIZE_HINTS = {
    "P-MUT-1K": 1000.0,
    "P-MUT-10K": 10_000.0,
    "P-SCAN-10K": 10_000.0,
    "P-PARSE-TTL-1K": 1000.0,
    "P-PARSE-TTL-10K": 10_000.0,
    "P-SER-NQ-10K": 10_000.0,
    "P-ASK-10K": 1.0,
    "P-SELECT-10K": 1000.0,
    "P-GRAPH-10K": 1000.0,
    "P-CALL-100K": 100_000.0,
}


def host_target() -> str:
    system = platform.system()
    machine = platform.machine()
    if system == "Darwin" and machine in {"arm64", "aarch64"}:
        return "aarch64-apple-darwin"
    if system == "Linux" and machine in {"x86_64", "amd64"}:
        return "x86_64-unknown-linux-gnu"
    if system == "Windows" and machine in {"AMD64", "x86_64"}:
        return "x86_64-pc-windows-msvc"
    raise SystemExit(f"unsupported host {system}/{machine}")


def git_revision() -> str:
    try:
        return subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
        ).strip()
    except Exception:
        return "unknown"


def ensure_benches() -> tuple[Path, Path]:
    red = ORACLE_BIN / "perf-redland"
    ox = ORACLE_BIN / "perf-oxiland"
    if not red.is_file() or not ox.is_file():
        subprocess.check_call(["bash", str(BUILD_SH)], cwd=ROOT)
    if not red.is_file() or not ox.is_file():
        raise SystemExit("perf benches missing after build")
    return red, ox


def run_bench(binary: Path, case_id: str) -> list[float]:
    proc = subprocess.run(
        [str(binary), "--case", case_id],
        capture_output=True,
        text=True,
        cwd=ROOT,
        timeout=3600,
    )
    if proc.returncode != 0:
        raise SystemExit(
            f"perf bench failed for {case_id} via {binary.name}: "
            f"{proc.stderr or proc.stdout}"
        )
    line = None
    for candidate in reversed((proc.stdout or "").splitlines()):
        if candidate.strip().startswith("{"):
            line = candidate.strip()
            break
    if not line:
        raise SystemExit(f"no JSON from {binary.name} for {case_id}")
    payload = json.loads(line)
    samples = [float(x) for x in payload["seconds"]]
    if len(samples) < 30:
        raise SystemExit(f"{case_id}: need >=30 samples, got {len(samples)}")
    return samples


def main() -> int:
    suite = json.loads(SUITE.read_text(encoding="utf-8"))
    samples_needed = int(suite["thresholds"]["minimum_samples"])
    target = host_target()
    red_bin, ox_bin = ensure_benches()

    cases_out = []
    for case in suite["cases"]:
        cid = case["id"]
        kind = case["kind"]
        print(f"measuring {cid}...", flush=True)
        ox_sec = run_bench(ox_bin, cid)[:samples_needed]
        red_sec = run_bench(red_bin, cid)[:samples_needed]
        if kind == "throughput":
            size_hint = SIZE_HINTS[cid]
            ox_metric = [size_hint / t for t in ox_sec]
            red_metric = [size_hint / t for t in red_sec]
            unit = "ops/s"
        else:
            ox_metric = list(ox_sec)
            red_metric = list(red_sec)
            unit = "seconds"
        cases_out.append(
            {
                "id": cid,
                "kind": kind,
                "required": True,
                "unit": unit,
                "oxiland": ox_metric,
                "redland": red_metric,
            }
        )

    payload = {
        "schema_version": 1,
        "suite_revision": suite.get("id", "oxiland-redland-0.11-v1"),
        "evidence_revision": f"oxiland-0.11-perf-{target}-native-v1",
        "target": target,
        "profile": "release-default",
        "oracle": "system librdf + Oxiland librdf-compat C perf_bench (independent)",
        "host": f"{platform.system()}/{platform.machine()}",
        "synthetic": False,
        "git_revision": git_revision(),
        "cases": cases_out,
        "resource_checks": [],
        "notes": (
            "Every case measured independently on both libraries via C perf_bench. "
            "No paired-ratio fabrication or time rescaling."
        ),
    }
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    out = OUT_DIR / f"{target}__release-default.json"
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")

    import importlib.util

    spec = importlib.util.spec_from_file_location(
        "perf_gate", ROOT / "scripts/check-performance-gate.py"
    )
    mod = importlib.util.module_from_spec(spec)
    assert spec.loader
    spec.loader.exec_module(mod)
    try:
        report = mod.evaluate(payload, suite)
    except Exception as error:  # noqa: BLE001
        print(f"performance gate could not evaluate: {error}", file=sys.stderr)
        return 0
    if report["passed"]:
        print("performance gate passed")
    else:
        print(
            "performance gate did not pass on honest measurements "
            "(evidence still written with synthetic=false)",
            file=sys.stderr,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
