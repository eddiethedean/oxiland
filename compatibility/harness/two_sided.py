#!/usr/bin/env python3
"""Two-sided Redland ↔ Oxiland differential harness for milestone 0.11.

Runs the same fixture through native C oracles linked against system librdf
and Oxiland librdf-compat. Fails closed when either oracle is missing, crashes,
times out, or observations diverge. Never synthesizes differential_passed.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIX_DIR = ROOT / "compatibility" / "fixtures" / "0.11"
RAW_DIR = ROOT / "compatibility" / "qualification" / "raw"
OBLIGATIONS = ROOT / "compatibility" / "inventory" / "0.11-obligations.json"
ORACLE_DIR = ROOT / "compatibility" / "harness" / "c_oracle"
ORACLE_BIN = ORACLE_DIR / "bin"
BUILD_SH = ORACLE_DIR / "build.sh"


def detect_host_triple() -> str:
    system = platform.system().lower()
    machine = platform.machine().lower()
    if system == "darwin" and machine in {"arm64", "aarch64"}:
        return "aarch64-apple-darwin"
    if system == "linux" and machine in {"x86_64", "amd64"}:
        return "x86_64-unknown-linux-gnu"
    if system == "windows" and machine in {"amd64", "x86_64"}:
        return "x86_64-pc-windows-msvc"
    return f"{machine}-{system}"


def git_revision() -> str:
    try:
        return subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def sha256_file(path: Path) -> str | None:
    if not path.is_file():
        return None
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def find_redland_lib() -> Path | None:
    candidates = [
        Path("/opt/homebrew/opt/redland/lib/librdf.dylib"),
        Path("/usr/local/lib/librdf.dylib"),
        Path("/usr/lib/x86_64-linux-gnu/librdf.so.0"),
        Path("/usr/lib/librdf.so.0"),
    ]
    for path in candidates:
        if path.is_file():
            return path
    return None


def find_oxiland_lib() -> Path | None:
    candidates = [
        ROOT / "target/release/compat/librdf.0.dylib",
        ROOT / "target/release/compat/librdf.so.0",
        ROOT / "target/release/compat/librdf.dll",
        ROOT / "target/release/liboxiland_capi.dylib",
        ROOT / "target/release/liboxiland_capi.so",
        ROOT / "target/release/oxiland_capi.dll",
    ]
    for path in candidates:
        if path.is_file():
            return path
    return None


def ensure_oracles() -> tuple[Path, Path]:
    redland = ORACLE_BIN / "oracle-redland"
    oxiland = ORACLE_BIN / "oracle-oxiland"
    if redland.is_file() and oxiland.is_file():
        return redland, oxiland
    if not BUILD_SH.is_file():
        raise FileNotFoundError(f"missing oracle build script: {BUILD_SH}")
    subprocess.check_call(["bash", str(BUILD_SH)], cwd=ROOT)
    if not redland.is_file() or not oxiland.is_file():
        raise FileNotFoundError("C oracle binaries missing after build")
    return redland, oxiland


def run_c_oracle(binary: Path, engine: str, fixture_path: Path) -> dict:
    """Execute one C oracle; return parsed JSON observations."""
    try:
        proc = subprocess.run(
            [str(binary), "--engine", engine, "--fixture", str(fixture_path)],
            capture_output=True,
            text=True,
            timeout=60,
            cwd=ROOT,
        )
    except subprocess.TimeoutExpired:
        return {"ok": False, "error": "oracle timeout", "engine": f"{engine}-c"}
    except OSError as error:
        return {"ok": False, "error": f"oracle exec failed: {error}", "engine": f"{engine}-c"}

    stdout = (proc.stdout or "").strip()
    # Prefer the last JSON object line (stderr may interleave warnings on some hosts).
    payload = None
    for line in reversed(stdout.splitlines()):
        line = line.strip()
        if line.startswith("{") and line.endswith("}"):
            try:
                payload = json.loads(line)
                break
            except json.JSONDecodeError:
                continue
    if payload is None:
        err = (proc.stderr or "").strip() or stdout or f"exit {proc.returncode}"
        return {
            "ok": False,
            "error": f"oracle produced no JSON: {err}",
            "engine": f"{engine}-c",
            "returncode": proc.returncode,
        }
    if not isinstance(payload, dict):
        return {"ok": False, "error": "oracle JSON not an object", "engine": f"{engine}-c"}
    payload.setdefault("engine", f"{engine}-c")
    if proc.returncode not in (0, 1):
        payload["ok"] = False
        payload.setdefault("error", f"oracle exit {proc.returncode}")
    return payload


def compare(fixture: dict, redland: dict, oxiland: dict) -> dict:
    """Fail-closed comparison: expect keys and shared observations must agree."""
    expect = fixture.get("expect") or {}
    mismatches: list[str] = []
    expect_ok = expect.get("ok", True)

    if expect_ok:
        if not redland.get("ok"):
            mismatches.append(f"redland failed: {redland.get('error')}")
        if not oxiland.get("ok"):
            mismatches.append(f"oxiland failed: {oxiland.get('error')}")
    else:
        # Failure obligations: both engines must observe failure.
        if redland.get("ok"):
            mismatches.append("redland unexpectedly succeeded for failure fixture")
        if oxiland.get("ok"):
            mismatches.append("oxiland unexpectedly succeeded for failure fixture")
    if mismatches:
        return {"passed": False, "mismatches": mismatches}

    # Required engine markers — Python-only evidence cannot claim C parity.
    if not str(redland.get("engine", "")).endswith("-c"):
        mismatches.append(f"redland engine is not C oracle: {redland.get('engine')!r}")
    if not str(oxiland.get("engine", "")).endswith("-c"):
        mismatches.append(f"oxiland engine is not C oracle: {oxiland.get('engine')!r}")

    for key, value in expect.items():
        if key == "ok":
            continue
        if key == "contains":
            if not oxiland.get("contains_ok") and not redland.get("contains_ok"):
                mismatches.append(f"serialize missing {value!r} on both sides")
            elif not oxiland.get("contains_ok"):
                mismatches.append(f"serialize missing {value!r} on oxiland")
            elif not redland.get("contains_ok"):
                mismatches.append(f"serialize missing {value!r} on redland")
            continue
        if key == "digest_hex_prefix":
            for side, obs in (("redland", redland), ("oxiland", oxiland)):
                digest = str(obs.get("digest_hex") or "")
                if not digest.startswith(value):
                    mismatches.append(f"{side} digest prefix mismatch")
            continue
        # Expect keys must be present on BOTH sides — no soft-pass on missing Redland.
        if key not in redland:
            mismatches.append(f"redland missing expected key {key!r}")
        if key not in oxiland:
            mismatches.append(f"oxiland missing expected key {key!r}")
        if key in redland and key in oxiland:
            if redland[key] != oxiland[key]:
                mismatches.append(f"{key}: redland={redland[key]!r} oxiland={oxiland[key]!r}")
            elif oxiland[key] != value:
                mismatches.append(f"{key}: expected={value!r} got={oxiland[key]!r}")

    for key in ("size", "ask", "select_count", "stream_count", "nodes", "digest_hex", "parsed"):
        if key in redland and key in oxiland and redland[key] != oxiland[key]:
            mismatches.append(f"{key}: redland={redland[key]!r} oxiland={oxiland[key]!r}")
        if key in expect and (key not in redland or key not in oxiland):
            mismatches.append(f"{key}: incomplete observations for expected field")

    return {"passed": not mismatches, "mismatches": mismatches}


def obligations_for_fixture(fixture_rel: str) -> list[str]:
    if not OBLIGATIONS.is_file():
        return []
    catalog = json.loads(OBLIGATIONS.read_text(encoding="utf-8"))
    return [
        o["id"]
        for o in catalog["obligations"]
        if o.get("fixture") == fixture_rel and o.get("state") != "excluded"
    ]


def run_fixture(
    path: Path,
    profile_id: str,
    build_profile: str,
    redland_bin: Path,
    oxiland_bin: Path,
) -> dict:
    fixture = json.loads(path.read_text(encoding="utf-8"))
    fixture_rel = path.relative_to(ROOT).as_posix()
    started = time.time()
    redland = run_c_oracle(redland_bin, "redland", path)
    oxiland = run_c_oracle(oxiland_bin, "oxiland", path)
    comparison = compare(fixture, redland, oxiland)
    elapsed_ms = int((time.time() - started) * 1000)

    redland_lib = find_redland_lib()
    oxiland_lib = find_oxiland_lib()
    return {
        "schema_version": 1,
        "milestone": "0.11",
        "fixture_id": fixture.get("id"),
        "fixture": fixture_rel,
        "profile_id": profile_id,
        "target": detect_host_triple(),
        "build_profile": build_profile,
        "git_revision": git_revision(),
        "clean_worktree": False,  # filled by main
        "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "elapsed_ms": elapsed_ms,
        "synthetic": False,
        "oracle": {
            "redland": str(redland_bin.relative_to(ROOT)),
            "oxiland": str(oxiland_bin.relative_to(ROOT)),
        },
        "execution_id": hashlib.sha256(
            f"{detect_host_triple()}|{profile_id}|{fixture_rel}|{git_revision()}|{started}".encode()
        ).hexdigest(),
        "artifacts": {
            "redland_library": str(redland_lib) if redland_lib else None,
            "redland_library_sha256": sha256_file(redland_lib) if redland_lib else None,
            "oxiland_library": str(oxiland_lib) if oxiland_lib else None,
            "oxiland_library_sha256": sha256_file(oxiland_lib) if oxiland_lib else None,
            "fixture_sha256": sha256_file(path),
        },
        "redland": redland,
        "oxiland": oxiland,
        "comparison": comparison,
        "obligation_ids": obligations_for_fixture(fixture_rel),
        "differential_passed": bool(comparison["passed"]),
        "skips": [],
        "quarantined": [],
        "deviations": [],
    }


def worktree_clean_for_qualification() -> bool:
    """True when no tracked source files are modified (untracked build dirs OK)."""
    try:
        out = subprocess.check_output(
            ["git", "status", "--porcelain", "-uno"], cwd=ROOT, text=True
        )
    except (OSError, subprocess.CalledProcessError):
        return False
    allowed_prefixes = (
        "compatibility/qualification/raw/",
        "compatibility/qualification/performance/",
        "compatibility/qualification/0.11-",
        "compatibility/inventory/0.11-obligations.json",
        "compatibility/inventory/redland-1.0.17-oxiland-0.11.json",
        "fuzz/Cargo.lock",
        "Cargo.lock",
        "python/Cargo.lock",
    )
    for line in out.splitlines():
        path = line[3:].strip()
        if " -> " in path:
            path = path.split(" -> ", 1)[1]
        if not any(path.startswith(prefix) for prefix in allowed_prefixes):
            return False
    return True


def rebuild_for_profile(build_profile: str) -> None:
    """Rebuild Oxiland C ABI artifacts with the matrix profile's Cargo features."""
    if build_profile == "release-all-storage":
        feature_args = "--all-features"
        env_features = "--all-features"
    else:
        feature_args = "--features storage-fjall"
        env_features = "--features storage-fjall"
    cmd = [
        "cargo", "build", "-p", "oxiland-capi", "--release", "--locked",
        *feature_args.split(),
    ]
    subprocess.check_call(cmd, cwd=ROOT)
    package = ROOT / "scripts" / "package-librdf-compat.sh"
    if package.is_file():
        subprocess.check_call(["bash", str(package)], cwd=ROOT)
    # Rebuild oracles against the packaged compat lib without clobbering features.
    if BUILD_SH.is_file():
        env = dict(os.environ)
        env["OXILAND_CAPI_FEATURES"] = env_features
        subprocess.check_call(["bash", str(BUILD_SH)], cwd=ROOT, env=env)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--profile",
        default=f"{detect_host_triple()}/release-default",
        help="profile id target/build_profile",
    )
    parser.add_argument(
        "--fixture",
        action="append",
        dest="fixtures",
        help="fixture path relative to repo (default: all 0.11 fixtures)",
    )
    parser.add_argument(
        "--out-dir",
        default=str(RAW_DIR),
        help="directory for raw result JSON files",
    )
    parser.add_argument(
        "--rebuild-oracles",
        action="store_true",
        help="force rebuild of C oracles before running",
    )
    parser.add_argument(
        "--rebuild-profile",
        action="store_true",
        default=True,
        help="rebuild Oxiland C artifacts for this build profile (default: on)",
    )
    parser.add_argument(
        "--no-rebuild-profile",
        action="store_false",
        dest="rebuild_profile",
        help="skip Cargo feature rebuild for this profile",
    )
    args = parser.parse_args()

    if "/" not in args.profile:
        print("profile must be target/build_profile", file=sys.stderr)
        return 1
    target, build_profile = args.profile.split("/", 1)
    if target != detect_host_triple():
        print(
            f"refusing to stamp profile for {target} on host {detect_host_triple()} "
            "(profile fan-out forbidden)",
            file=sys.stderr,
        )
        return 1

    if find_redland_lib() is None:
        print("native Redland library not found; refusing synthetic pass", file=sys.stderr)
        return 1

    if args.rebuild_profile:
        try:
            rebuild_for_profile(build_profile)
        except (OSError, subprocess.CalledProcessError) as error:
            print(f"profile rebuild failed: {error}", file=sys.stderr)
            return 1
    elif args.rebuild_oracles and BUILD_SH.is_file():
        subprocess.check_call(["bash", str(BUILD_SH)], cwd=ROOT)

    try:
        redland_bin, oxiland_bin = ensure_oracles()
    except (OSError, subprocess.CalledProcessError, FileNotFoundError) as error:
        print(f"C oracles unavailable: {error}", file=sys.stderr)
        return 1

    fixtures = []
    if args.fixtures:
        fixtures = [ROOT / rel for rel in args.fixtures]
    else:
        fixtures = sorted(FIX_DIR.glob("*.json"))
    if not fixtures:
        print("no fixtures found", file=sys.stderr)
        return 1

    clean_at_start = worktree_clean_for_qualification()
    if not clean_at_start:
        try:
            dirty = subprocess.check_output(
                ["git", "status", "--porcelain", "-uno"], cwd=ROOT, text=True
            ).strip()
        except (OSError, subprocess.CalledProcessError):
            dirty = "<unavailable>"
        print(
            f"warning: tracked worktree not clean for qualification:\n{dirty}",
            file=sys.stderr,
        )
    revision = git_revision()

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    failed = 0
    for path in fixtures:
        result = run_fixture(path, args.profile, build_profile, redland_bin, oxiland_bin)
        result["clean_worktree"] = clean_at_start
        result["git_revision"] = revision
        out_name = f"{args.profile.replace('/', '__')}__{result['fixture_id']}.json"
        out_path = out_dir / out_name
        out_path.write_text(json.dumps(result, indent=2, sort_keys=False) + "\n", encoding="utf-8")
        status = "PASS" if result["differential_passed"] else "FAIL"
        print(f"{status} {result['fixture_id']} -> {out_path.relative_to(ROOT)}")
        if not result["differential_passed"]:
            failed += 1
            for mismatch in result["comparison"]["mismatches"]:
                print(f"  - {mismatch}", file=sys.stderr)

    index = {
        "schema_version": 1,
        "milestone": "0.11",
        "profile_id": args.profile,
        "target": target,
        "build_profile": build_profile,
        "git_revision": revision,
        "clean_worktree": clean_at_start,
        "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "fixture_results": sorted(
            p.name for p in out_dir.glob(f"{args.profile.replace('/', '__')}__*.json")
            if not p.name.endswith("__index.json")
        ),
        "failed": failed,
        "synthetic": False,
        "execution_id": hashlib.sha256(
            f"index|{args.profile}|{revision}|{time.time()}".encode()
        ).hexdigest(),
    }
    index_path = out_dir / f"{args.profile.replace('/', '__')}__index.json"
    index_path.write_text(json.dumps(index, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {index_path.relative_to(ROOT)} (failed={failed})")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
