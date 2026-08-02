#!/usr/bin/env python3
"""Freeze the Oxiland 0.11 Redland baseline denominator.

Captures checksummed public headers (from compatibility/baseline/headers/),
export dumps, library identity, Raptor/Rasqal pins, and rdfproc CLI surface
into compatibility/baseline/0.11-baseline-manifest.json.
"""

from __future__ import annotations

import hashlib
import json
import platform
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "compatibility" / "baseline"
HEADERS = BASELINE / "headers"
EXPORTS = BASELINE / "exports"
MANIFEST = BASELINE / "0.11-baseline-manifest.json"
TARBALL_SHA = (
    "de1847f7b59021c16bdc72abb4d8e2d9187cd6124d69156f3326dd34ee043681"
)
RDFPROC_WORKFLOWS = [
    "parse",
    "print",
    "serialize",
    "query",
    "ask",
    "add",
    "remove",
    "contains",
    "contexts",
    "size",
]


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def detect_host_triple() -> str:
    system = platform.system().lower()
    machine = platform.machine().lower()
    if system == "darwin" and machine in {"arm64", "aarch64"}:
        return "aarch64-apple-darwin"
    if system == "darwin" and machine in {"x86_64", "amd64"}:
        return "x86_64-apple-darwin"
    if system == "linux" and machine in {"x86_64", "amd64"}:
        return "x86_64-unknown-linux-gnu"
    if system == "windows" and machine in {"amd64", "x86_64"}:
        return "x86_64-pc-windows-msvc"
    return f"{machine}-{system}"


def parse_exports(path: Path) -> list[dict[str, str]]:
    symbols: list[dict[str, str]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        name = line.strip()
        if not name.startswith("librdf_"):
            continue
        # Heuristic: known data symbols from Redland exports.
        kind = (
            "data"
            if name
            in {
                "librdf_copyright_string",
                "librdf_home_url_string",
                "librdf_version_decimal",
                "librdf_version_major",
                "librdf_version_minor",
                "librdf_version_release",
                "librdf_version_string",
                "librdf_short_copyright_string",
                "librdf_license_string",
                "librdf_concept_concepts",
                "librdf_concept_ms_namespace",
                "librdf_concept_schemas_namespace",
            }
            or name.endswith("_string")
            or "version_" in name
            or name.startswith("librdf_concept_")
            else "function"
        )
        symbols.append({"name": name, "kind": kind})
    return symbols


def public_declarations(headers_dir: Path) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for header in sorted(headers_dir.glob("*.h")):
        text = header.read_text(encoding="utf-8", errors="replace")
        rows.append(
            {
                "path": f"compatibility/baseline/headers/{header.name}",
                "sha256": sha256_file(header),
                "role": "public",
                "bytes": str(header.stat().st_size),
                "lines": str(text.count("\n") + 1),
            }
        )
    return rows


def main() -> int:
    if not HEADERS.is_dir() or not any(HEADERS.glob("*.h")):
        print(
            "build-0.11-baseline: missing compatibility/baseline/headers/*.h "
            "(copy from a pinned Redland install first)",
            file=sys.stderr,
        )
        return 1

    host = detect_host_triple()
    export_path = EXPORTS / f"librdf-{host}.txt"
    if not export_path.is_file():
        print(
            f"build-0.11-baseline: missing export dump {export_path.relative_to(ROOT)}",
            file=sys.stderr,
        )
        return 1

    lib_sha_path = EXPORTS / f"librdf-{host}.sha256"
    lib_sha = (
        lib_sha_path.read_text(encoding="utf-8").split()[0]
        if lib_sha_path.is_file()
        else None
    )

    try:
        git_revision = subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        git_revision = "unknown"

    symbols = parse_exports(export_path)
    headers = public_declarations(HEADERS)
    header_index_sha = sha256_file(BASELINE / "headers.sha256") if (
        BASELINE / "headers.sha256"
    ).is_file() else hashlib.sha256(
        "\n".join(f"{h['sha256']}  {Path(h['path']).name}" for h in headers).encode()
    ).hexdigest()

    manifest = {
        "schema_version": 1,
        "milestone": "0.11",
        "frozen_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "generated_by": "scripts/build-0.11-baseline.py",
        "git_revision": git_revision,
        "redland": {
            "api": "1.0.17",
            "manual": "1.0.18",
            "tarball_url": "https://download.librdf.org/source/redland-1.0.17.tar.gz",
            "tarball_sha256": TARBALL_SHA,
            "raptor": {
                "version": "2.0.16",
                "notes": "Pinned packaged Raptor2 used by Homebrew/reference installs",
            },
            "rasqal": {
                "version": "0.9.33",
                "notes": "Pinned packaged Rasqal used by Homebrew/reference installs",
            },
            "configure_flags": [
                "--with-raptor=system",
                "--with-rasqal=system",
                "--enable-release",
            ],
            "soname": {
                "darwin": "librdf.0.dylib",
                "linux": "librdf.so.0",
                "windows": "librdf-0.dll",
            },
            "compatibility_version": "1.0.0",
        },
        "oxigraph_version": "0.5.9",
        "headers": headers,
        "header_index_sha256": header_index_sha,
        "exports": {
            "host_triple": host,
            "path": str(export_path.relative_to(ROOT)),
            "artifact_sha256": lib_sha,
            "symbol_count": len(symbols),
            "symbols": symbols,
        },
        "cli": {
            "rdfproc_workflows": RDFPROC_WORKFLOWS,
            "fixture_refs": [
                "compatibility/fixtures/0.11/cli-parse-ask.json",
                "compatibility/fixtures/0.11/world-lifecycle.json",
            ],
        },
        "format_matrix": "compatibility/baseline/format-matrix.json",
        "target_matrix": "compatibility/qualification/0.11-matrix.json",
        "obligation_catalog": "compatibility/inventory/0.11-obligations.json",
        "notes": (
            "0.11 denominator freeze: public headers, exports, library identity, "
            "Raptor/Rasqal pins, and rdfproc workflows. Independent Raptor/Rasqal "
            "APIs remain out of scope."
        ),
    }

    MANIFEST.write_text(
        json.dumps(manifest, indent=2, sort_keys=False) + "\n", encoding="utf-8"
    )
    print(f"wrote {MANIFEST.relative_to(ROOT)}")
    print(f"headers: {len(headers)}")
    print(f"exports: {len(symbols)} ({host})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
