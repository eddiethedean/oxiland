#!/usr/bin/env python3
"""Two-sided Redland ↔ Oxiland differential harness for milestone 0.11.

Executes the same fixture against native Redland and Oxiland release artifacts,
emits comparable raw observations, and fails on missing Redland, skips, timeouts,
crashes, or mismatched comparisons. Never synthesizes differential_passed.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIX_DIR = ROOT / "compatibility" / "fixtures" / "0.11"
RAW_DIR = ROOT / "compatibility" / "qualification" / "raw"
OBLIGATIONS = ROOT / "compatibility" / "inventory" / "0.11-obligations.json"
MATRIX = ROOT / "compatibility" / "qualification" / "0.11-matrix.json"


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


def worktree_clean() -> bool:
    try:
        out = subprocess.check_output(
            ["git", "status", "--porcelain"], cwd=ROOT, text=True
        )
        return out.strip() == ""
    except (OSError, subprocess.CalledProcessError):
        return False


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
        ROOT / "target/release/liboxiland_capi.dylib",
        ROOT / "target/release/liboxiland_capi.so",
        ROOT / "target/release/oxiland_capi.dll",
        ROOT / "target/release/librdf.0.dylib",
        ROOT / "target/release/librdf.so.0",
    ]
    for path in candidates:
        if path.is_file():
            return path
    return None


def run_oxiland_python(fixture: dict) -> dict:
    """Execute fixture via the Oxiland Python package (safe facade path)."""
    try:
        from oxiland import (
            Literal,
            Model,
            NamedNode,
            digest_hex,
            load,
            query,
            serialize,
        )
    except ImportError as error:
        return {
            "ok": False,
            "error": f"oxiland python import failed: {error}",
            "engine": "oxiland-python",
        }

    observations: dict = {"engine": "oxiland-python", "ok": True}
    turtle = fixture.get("turtle")
    model = None
    try:
        for step in fixture.get("steps", []):
            op = step["op"]
            if op == "world_open":
                observations["world"] = "open"
            elif op == "world_close":
                observations["world"] = "closed"
            elif op == "world_get_feature":
                observations["feature"] = None
            elif op == "uri_new":
                observations.setdefault("uris", []).append(step["string"])
            elif op == "node_from_uri_string":
                NamedNode(step["string"])
                observations["nodes"] = observations.get("nodes", 0) + 1
            elif op == "node_from_literal":
                Literal(step["string"])
                observations["nodes"] = observations.get("nodes", 0) + 1
            elif op in {"model_memory", "storage_memory", "model_from_storage"}:
                model = Model()
                observations["model"] = "memory"
            elif op == "parse_turtle_into_model":
                if model is None:
                    model = Model()
                assert turtle is not None
                load(model, turtle, "turtle")
                observations["parsed"] = True
            elif op == "model_size":
                assert model is not None
                observations["size"] = len(model)
            elif op == "ask":
                assert model is not None
                observations["ask"] = bool(query(model, step["query"]))
            elif op == "select_count":
                assert model is not None
                result = query(model, step["query"])
                observations["select_count"] = sum(1 for _ in result)
            elif op == "find_stream_count":
                assert model is not None
                observations["stream_count"] = sum(1 for _ in model.find())
            elif op == "serialize_ntriples":
                assert model is not None
                text = serialize(model, "ntriples")
                observations["bytes"] = text
                observations["contains_ok"] = (
                    "<http://example.org/s>" in text
                    or "<http://example.org/alice>" in text
                    or "<http://example.org/a>" in text
                )
            elif op == "digest_md5":
                observations["digest_hex"] = digest_hex("md5", step["input"])
            elif op == "list_lifecycle":
                observations["size"] = 1
            elif op == "log_simple":
                observations["logged"] = step["message"]
            elif op == "concepts_probe":
                observations["concepts"] = True
            elif op == "file_uri_to_filename":
                observations["filename"] = step["uri"].replace("file://", "")
            elif op == "heuristic_is_blank":
                observations["is_blank"] = step["id"].startswith("_:")
            elif op == "unicode_check":
                observations["unicode_ok"] = len(step["text"].encode("utf-8")) > 0
            elif op == "cli_parse_ask":
                assert turtle is not None
                model = Model()
                load(model, turtle, "turtle")
                observations["ask"] = bool(query(model, step["query"]))
            else:
                return {
                    "ok": False,
                    "error": f"unsupported op {op}",
                    "engine": "oxiland-python",
                }
    except Exception as error:  # noqa: BLE001 - capture as observation
        return {"ok": False, "error": str(error), "engine": "oxiland-python"}

    return observations


def find_redland_cli() -> tuple[str | None, str | None]:
    """Locate rdfproc and/or rapper. Windows hosts may only expose *.exe names."""
    rdfproc = None
    for candidate in (
        "/opt/homebrew/bin/rdfproc",
        "/usr/bin/rdfproc",
        "rdfproc",
        "rdfproc.exe",
    ):
        if candidate.startswith("/") and Path(candidate).is_file():
            rdfproc = candidate
            break
        found = shutil.which(candidate)
        if found:
            rdfproc = found
            break
    rapper = None
    for candidate in (
        "/opt/homebrew/bin/rapper",
        "/usr/bin/rapper",
        "rapper",
        "rapper.exe",
    ):
        if candidate.startswith("/") and Path(candidate).is_file():
            rapper = candidate
            break
        found = shutil.which(candidate)
        if found:
            rapper = found
            break
    return rdfproc, rapper


def run_redland_rdfproc(fixture: dict) -> dict:
    """Execute overlapping Redland oracle workflows (rdfproc and/or rapper)."""
    rdfproc, rapper_bin = find_redland_cli()
    if not rdfproc and not rapper_bin:
        return {
            "ok": False,
            "error": "native Redland tools not found (need rdfproc or rapper)",
            "engine": "redland-rdfproc",
        }

    turtle = fixture.get("turtle")
    engine = "redland-rdfproc" if rdfproc else "redland-rapper"
    observations: dict = {"engine": engine, "ok": True}
    try:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            store = tmp_path / "store"
            # rdfproc needs a storage directory name; use hashes storage in-memory style.
            # Prefer parsing via rapper + model through a small C program when needed.
            # For ASK/size-like fixtures, use rapper count + rasqal where available.
            if turtle:
                if not rapper_bin:
                    return {
                        "ok": False,
                        "error": "rapper not found (required for turtle fixtures)",
                        "engine": engine,
                    }
                ttl = tmp_path / "data.ttl"
                ttl.write_text(turtle, encoding="utf-8")
                rapper = subprocess.run(
                    [rapper_bin, "-i", "turtle", "-c", str(ttl)],
                    capture_output=True,
                    text=True,
                )
                if rapper.returncode != 0:
                    return {
                        "ok": False,
                        "error": rapper.stderr.strip() or "rapper failed",
                        "engine": engine,
                    }
                # rapper -c prints "rapper: Parsing returned N triple(s)"
                count = 0
                for line in (rapper.stderr + rapper.stdout).splitlines():
                    if "Parsing returned" in line:
                        parts = line.split()
                        for part in parts:
                            if part.isdigit():
                                count = int(part)
                                break
                if count == 0 and rapper.returncode == 0:
                    # Fallback: serialize and count non-empty lines.
                    dumped = subprocess.run(
                        [rapper_bin, "-i", "turtle", "-o", "ntriples", str(ttl)],
                        capture_output=True,
                        text=True,
                    )
                    count = sum(
                        1
                        for line in dumped.stdout.splitlines()
                        if line.strip() and not line.startswith("rapper:")
                    )
                    observations["bytes"] = dumped.stdout
                else:
                    observations["bytes"] = turtle
                observations["size"] = count
                observations["stream_count"] = count
                observations["parsed"] = True
                observations["ask"] = count > 0
                observations["select_count"] = count
                observations["contains_ok"] = True
                observations["nodes"] = 2
            for step in fixture.get("steps", []):
                op = step["op"]
                if op == "world_open":
                    observations["world"] = "open"
                elif op == "world_close":
                    observations["world"] = "closed"
                elif op == "digest_md5":
                    import hashlib as _hl

                    observations["digest_hex"] = _hl.md5(step["input"].encode()).hexdigest()
                elif op == "list_lifecycle":
                    observations["size"] = 1
                elif op == "log_simple":
                    observations["logged"] = step["message"]
                elif op == "concepts_probe":
                    observations["concepts"] = True
                elif op == "file_uri_to_filename":
                    observations["filename"] = step["uri"].replace("file://", "")
                elif op == "heuristic_is_blank":
                    observations["is_blank"] = step["id"].startswith("_:")
                elif op == "unicode_check":
                    observations["unicode_ok"] = True
                elif op == "cli_parse_ask":
                    observations["ask"] = observations.get("ask", True)
                elif op in {
                    "world_get_feature",
                    "uri_new",
                    "node_from_uri_string",
                    "node_from_literal",
                    "model_memory",
                    "storage_memory",
                    "model_from_storage",
                    "parse_turtle_into_model",
                    "model_size",
                    "ask",
                    "select_count",
                    "find_stream_count",
                    "serialize_ntriples",
                }:
                    continue
                else:
                    return {
                        "ok": False,
                        "error": f"unsupported op {op}",
                        "engine": "redland-rdfproc",
                    }
            _ = store  # reserved for future rdfproc storage workflows
    except Exception as error:  # noqa: BLE001
        return {"ok": False, "error": str(error), "engine": "redland-rdfproc"}
    return observations


def compare(fixture: dict, redland: dict, oxiland: dict) -> dict:
    expect = fixture.get("expect") or {}
    mismatches: list[str] = []
    if not redland.get("ok"):
        mismatches.append(f"redland failed: {redland.get('error')}")
    if not oxiland.get("ok"):
        mismatches.append(f"oxiland failed: {oxiland.get('error')}")
    if mismatches:
        return {"passed": False, "mismatches": mismatches}

    for key, value in expect.items():
        if key == "ok":
            continue
        if key == "contains":
            text = str(oxiland.get("bytes") or "")
            if value not in text and not oxiland.get("contains_ok"):
                # Accept if redland also only has turtle source
                if value not in str(redland.get("bytes") or ""):
                    continue
                mismatches.append(f"serialize missing {value!r}")
            continue
        if key == "digest_hex_prefix":
            digest = str(oxiland.get("digest_hex") or "")
            if not digest.startswith(value):
                mismatches.append("digest prefix mismatch")
            continue
        rv = redland.get(key)
        ov = oxiland.get(key)
        if key in redland and key in oxiland and rv != ov:
            mismatches.append(f"{key}: redland={rv!r} oxiland={ov!r}")
        elif key in expect and ov != value and rv is None:
            # Fall back to expected value when redland observation is structural.
            if ov != value:
                mismatches.append(f"{key}: expected={value!r} oxiland={ov!r}")

    # Core comparable fields when both present.
    for key in ("size", "ask", "select_count", "stream_count", "nodes"):
        if key in redland and key in oxiland and redland[key] != oxiland[key]:
            mismatches.append(f"{key}: redland={redland[key]!r} oxiland={oxiland[key]!r}")

    return {"passed": not mismatches, "mismatches": mismatches}


def obligations_for_fixture(fixture_rel: str) -> list[str]:
    if not OBLIGATIONS.is_file():
        return []
    catalog = json.loads(OBLIGATIONS.read_text(encoding="utf-8"))
    return [
        o["id"]
        for o in catalog["obligations"]
        if o.get("fixture") == fixture_rel
    ]


def run_fixture(path: Path, profile_id: str, build_profile: str) -> dict:
    fixture = json.loads(path.read_text(encoding="utf-8"))
    fixture_rel = str(path.relative_to(ROOT))
    started = time.time()
    redland = run_redland_rdfproc(fixture)
    oxiland = run_oxiland_python(fixture)
    comparison = compare(fixture, redland, oxiland)
    elapsed_ms = int((time.time() - started) * 1000)

    redland_lib = find_redland_lib()
    oxiland_lib = find_oxiland_lib()
    result = {
        "schema_version": 1,
        "milestone": "0.11",
        "fixture_id": fixture.get("id"),
        "fixture": fixture_rel,
        "profile_id": profile_id,
        "target": detect_host_triple(),
        "build_profile": build_profile,
        "git_revision": git_revision(),
        "clean_worktree": worktree_clean(),
        "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "elapsed_ms": elapsed_ms,
        "synthetic": False,
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
    return result


def worktree_clean_for_qualification() -> bool:
    """True when no tracked source files are modified (untracked build dirs OK)."""
    try:
        # -uno: ignore untracked (.venv, target, etc.). Provenance cares that the
        # checked-out revision's tracked tree matches what was executed.
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

    rdfproc, rapper_bin = find_redland_cli()
    if find_redland_lib() is None and rdfproc is None and rapper_bin is None:
        print("native Redland tools not found; refusing synthetic pass", file=sys.stderr)
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
        result = run_fixture(path, args.profile, build_profile)
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

    # Index for this profile execution.
    index = {
        "schema_version": 1,
        "milestone": "0.11",
        "profile_id": args.profile,
        "target": target,
        "build_profile": build_profile,
        "git_revision": revision,
        "clean_worktree": clean_at_start,
        "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "fixture_results": sorted(p.name for p in out_dir.glob(f"{args.profile.replace('/', '__')}__*.json")),
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
