#!/usr/bin/env python3
"""Differential smoke: Oxiland digests vs hashlib known vectors."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    cases = [
        ("md5", b"", "d41d8cd98f00b204e9800998ecf8427e"),
        ("sha1", b"abc", "a9993e364706816aba3e25717850c26c9cd0d89d"),
        ("sha256", b"abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
    ]
    results = []
    status = "pass"
    for algorithm, data, expected in cases:
        native = {
            "md5": hashlib.md5,
            "sha1": hashlib.sha1,
            "sha256": hashlib.sha256,
        }[algorithm](data).hexdigest()
        ok = native == expected
        if not ok:
            status = "fail"
        results.append(
            {
                "algorithm": algorithm,
                "expected": expected,
                "native_hashlib": native,
                "status": "pass" if ok else "fail",
            }
        )

    cargo = subprocess.run(
        [
            "cargo",
            "test",
            "--test",
            "utility",
            "digest_hex_known_vectors",
            "--",
            "--exact",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if cargo.returncode != 0:
        status = "fail"

    report = {
        "schema_version": 1,
        "suite": "oxiland-utility-digest-smoke",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "classification": "hashlib-vector",
        "cases": results,
        "cargo_exit_code": cargo.returncode,
    }
    out = ROOT / "compatibility" / "harness" / "utility-digest-smoke-result.json"
    out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if status == "pass" else 1


if __name__ == "__main__":
    sys.exit(main())
