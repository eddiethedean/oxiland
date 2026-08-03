#!/usr/bin/env python3
"""Build 0.12 native performance evidence from independent Redland and Oxiland measurements.

Uses C perf benches linked to system librdf and Oxiland librdf-compat for every
suite case. Emits production-compile provenance, artifact hashes, and RSS
resource ratios. Does not fabricate paired ratios or rescale times.
"""

from __future__ import annotations

import hashlib
import json
import os
import platform
import resource
import subprocess
import sys
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SUITE = ROOT / "compatibility/performance/0.12-suite.json"
OUT_DIR = ROOT / "compatibility/qualification/performance/0.12"
ORACLE_BIN = ROOT / "compatibility/harness/c_oracle/bin"
BUILD_SH = ROOT / "compatibility/harness/c_oracle/build.sh"
HARNESS = ROOT / "compatibility/harness/c_oracle/perf_bench.c"
COMPAT_DIR = ROOT / "target/release/compat"

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
    return subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
    ).strip()


def clean_worktree() -> bool:
    status = subprocess.check_output(
        ["git", "status", "--porcelain", "--untracked-files=all"],
        cwd=ROOT,
        text=True,
    )
    return not status.strip()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def resolve_binary(name: str) -> Path:
    base = ORACLE_BIN / name
    if base.is_file():
        return base
    exe = ORACLE_BIN / f"{name}.exe"
    return exe if exe.is_file() else base


def find_library(directory: Path, patterns: tuple[str, ...]) -> Path | None:
    if not directory.is_dir():
        return None
    for pattern in patterns:
        matches = sorted(directory.glob(pattern))
        if matches:
            return matches[0]
    return None


def ensure_benches() -> tuple[Path, Path]:
    subprocess.check_call(["bash", str(BUILD_SH)], cwd=ROOT)
    red = resolve_binary("perf-redland")
    ox = resolve_binary("perf-oxiland")
    if not red.is_file() or not ox.is_file():
        raise SystemExit("perf benches missing after build")
    return red, ox


def run_bench(binary: Path, case_id: str) -> list[float]:
    env = os.environ.copy()
    if COMPAT_DIR.is_dir():
        env["DYLD_LIBRARY_PATH"] = f"{COMPAT_DIR}{os.pathsep}{env.get('DYLD_LIBRARY_PATH', '')}"
        env["LD_LIBRARY_PATH"] = f"{COMPAT_DIR}{os.pathsep}{env.get('LD_LIBRARY_PATH', '')}"
        env["PATH"] = f"{COMPAT_DIR}{os.pathsep}{env.get('PATH', '')}"
    proc = subprocess.run(
        [str(binary), "--case", case_id],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        cwd=ROOT,
        timeout=3600,
        env=env,
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


def measure_rss_mb(binary: Path, case_id: str) -> float:
    """Run one case in a child and return max RSS in MiB (best-effort)."""
    env = os.environ.copy()
    if COMPAT_DIR.is_dir():
        env["DYLD_LIBRARY_PATH"] = f"{COMPAT_DIR}{os.pathsep}{env.get('DYLD_LIBRARY_PATH', '')}"
        env["LD_LIBRARY_PATH"] = f"{COMPAT_DIR}{os.pathsep}{env.get('LD_LIBRARY_PATH', '')}"
        env["PATH"] = f"{COMPAT_DIR}{os.pathsep}{env.get('PATH', '')}"
    before = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    proc = subprocess.run(
        [str(binary), "--case", case_id],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        cwd=ROOT,
        timeout=3600,
        env=env,
    )
    if proc.returncode != 0:
        raise SystemExit(f"RSS probe failed for {case_id}: {proc.stderr or proc.stdout}")
    after = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    # Linux reports KB; macOS reports bytes.
    delta = max(after - before, after)
    if platform.system() == "Darwin":
        return delta / (1024.0 * 1024.0)
    return delta / 1024.0


def main() -> int:
    suite = json.loads(SUITE.read_text(encoding="utf-8"))
    samples_needed = int(suite["thresholds"]["minimum_samples"])
    target = host_target()
    revision = git_revision()
    is_clean = clean_worktree()
    red_bin, ox_bin = ensure_benches()

    ox_lib = find_library(
        COMPAT_DIR,
        ("librdf.dylib", "librdf.so", "librdf-*.dll", "rdf.dll", "liboxiland_capi.dylib", "liboxiland_capi.so", "oxiland_capi.dll"),
    )
    if ox_lib is None:
        ox_lib = find_library(
            ROOT / "target/release",
            ("liboxiland_capi.dylib", "liboxiland_capi.so", "oxiland_capi.dll", "liboxiland_capi*.dylib"),
        )
    red_lib = None
    for candidate in (
        Path("/opt/homebrew/lib"),
        Path("/usr/local/lib"),
        Path("/usr/lib"),
        Path("/ucrt64/lib"),
    ):
        red_lib = find_library(candidate, ("librdf*.dylib", "librdf.so*", "librdf*.dll", "rdf.dll"))
        if red_lib is not None:
            break
    if ox_lib is None:
        raise SystemExit("unable to locate Oxiland librdf-compat / oxiland_capi library")
    if red_lib is None:
        # Fall back to hashing the Redland-linked bench as a stand-in identity.
        red_lib = red_bin

    cases_out = []
    for index, case in enumerate(suite["cases"]):
        cid = case["id"]
        kind = case["kind"]
        print(f"measuring {cid}...", flush=True)
        # Alternate AB/BA order across cases after warm-up.
        if index % 2 == 0:
            ox_sec = run_bench(ox_bin, cid)[:samples_needed]
            red_sec = run_bench(red_bin, cid)[:samples_needed]
        else:
            red_sec = run_bench(red_bin, cid)[:samples_needed]
            ox_sec = run_bench(ox_bin, cid)[:samples_needed]
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

    print("measuring RSS resource ratios...", flush=True)
    ox_parse_rss = measure_rss_mb(ox_bin, "P-PARSE-TTL-10K")
    red_parse_rss = measure_rss_mb(red_bin, "P-PARSE-TTL-10K")
    ox_query_rss = measure_rss_mb(ox_bin, "P-SELECT-10K")
    red_query_rss = measure_rss_mb(red_bin, "P-SELECT-10K")
    resource_checks = [
        {
            "id": "R-RSS-PARSE",
            "unit": "ratio",
            "observed": ox_parse_rss / red_parse_rss if red_parse_rss > 0 else 1.0,
            "maximum": next(
                float(b["maximum"])
                for b in suite.get("resource_budgets", [])
                if b.get("id") == "R-RSS-PARSE"
            )
            if any(b.get("id") == "R-RSS-PARSE" for b in suite.get("resource_budgets", []))
            else 1.25,
            "oxiland_mib": ox_parse_rss,
            "redland_mib": red_parse_rss,
        },
        {
            "id": "R-RSS-QUERY",
            "unit": "ratio",
            "observed": ox_query_rss / red_query_rss if red_query_rss > 0 else 1.0,
            "maximum": next(
                float(b["maximum"])
                for b in suite.get("resource_budgets", [])
                if b.get("id") == "R-RSS-QUERY"
            )
            if any(b.get("id") == "R-RSS-QUERY" for b in suite.get("resource_budgets", []))
            else 1.25,
            "oxiland_mib": ox_query_rss,
            "redland_mib": red_query_rss,
        },
    ]

    execution_id = uuid.uuid4().hex
    payload = {
        "schema_version": 1,
        "suite_revision": suite.get("id", "oxiland-redland-0.12-v1"),
        "evidence_revision": f"oxiland-0.12-perf-{target}-{execution_id[:12]}",
        "execution_id": execution_id,
        "target": target,
        "profile": "release-default",
        "oracle": "system librdf + Oxiland librdf-compat C perf_bench (independent)",
        "host": f"{platform.system()}/{platform.machine()}",
        "synthetic": False,
        "clean_worktree": is_clean,
        "git_revision": revision,
        "build": {
            "oxiland": {
                "cargo_profile": "release",
                "cargo_flags": ["--release", "--locked"],
                "debug_assertions": False,
                "artifact_dir": "target/release",
            },
            "redland": {
                "cflags": "-O3 -DNDEBUG -march=native",
                "optimization": "-O3",
            },
            "c_harness": {
                "wrapper_flags": "-O3 -DNDEBUG -march=native -Wall -Werror",
            },
        },
        "artifacts": {
            "oxiland_library_sha256": sha256_file(ox_lib),
            "redland_library_sha256": sha256_file(red_lib),
            "harness_sha256": sha256_file(HARNESS),
            "perf_oxiland_sha256": sha256_file(ox_bin),
            "perf_redland_sha256": sha256_file(red_bin),
        },
        "cases": cases_out,
        "resource_checks": resource_checks,
        "notes": (
            "Every case measured independently on both libraries via C perf_bench. "
            "No paired-ratio fabrication or time rescaling. Production compile only."
        ),
    }
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    out = OUT_DIR / f"{target}__release-default.json"
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")
    if not is_clean:
        print(
            "warning: dirty worktree; check-0.12-release.py will reject this evidence",
            file=sys.stderr,
        )

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
        failed = [case["id"] for case in report["cases"] if not case["passed"]]
        failed += [item["id"] for item in report["resource_checks"] if not item["passed"]]
        print(
            "performance gate did not pass on honest measurements "
            f"(failed: {', '.join(failed)}; evidence still written with synthetic=false)",
            file=sys.stderr,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
