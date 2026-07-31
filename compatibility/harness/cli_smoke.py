#!/usr/bin/env python3
"""Smoke: oxiland-cli parse + find against a Turtle fixture."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "compatibility" / "harness" / "cli-smoke-result.json"
FIXTURE = ROOT / "compatibility" / "fixtures" / "io" / "smoke.ttl"


def main() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        store = Path(tmp) / "store"
        parse = subprocess.run(
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "oxiland-cli",
                "--",
                "-n",
                "-s",
                "fjall",
                str(store),
                "parse",
                str(FIXTURE),
                "--syntax",
                "turtle",
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        find = subprocess.run(
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "oxiland-cli",
                "--",
                "-s",
                "fjall",
                str(store),
                "find",
                "-",
                "-",
                "-",
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
    status = "pass" if parse.returncode == 0 and find.returncode == 0 else "fail"
    result = {
        "schema_version": 1,
        "suite": "oxiland-cli-smoke",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "parse_exit_code": parse.returncode,
        "find_exit_code": find.returncode,
        "find_stdout_len": len(find.stdout),
    }
    OUT.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))
    return 0 if status == "pass" else 1


if __name__ == "__main__":
    sys.exit(main())
