#!/usr/bin/env python3
"""Native Raptor oracle smoke for Oxiland 0.2 I/O fixtures."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIXTURE_DIR = ROOT / "compatibility" / "fixtures" / "io"
FIXTURE = FIXTURE_DIR / "smoke.ttl"
OUT = ROOT / "compatibility" / "harness" / "oracle-smoke-result.json"


def main() -> int:
    rapper = shutil.which("rapper")
    result = {
        "schema_version": 1,
        "suite": "oxiland-io-oracle-smoke",
        "fixture": "io.smoke.turtle.one-triple",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "platform": sys.platform,
    }

    if rapper is None:
        result.update(
            {
                "status": "skip",
                "reason": "rapper not installed; install raptor2-utils/libraptor2",
            }
        )
        OUT.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(result, indent=2))
        return 0

    version = subprocess.run(
        [rapper, "-v"],
        check=False,
        capture_output=True,
        text=True,
    )
    parse = subprocess.run(
        [rapper, "-i", "turtle", "-c", str(FIXTURE)],
        check=False,
        capture_output=True,
        text=True,
    )
    result.update(
        {
            "status": "pass" if parse.returncode == 0 else "fail",
            "oracle": {
                "binary": rapper,
                "version_stdout": version.stdout.strip() or version.stderr.strip(),
            },
            "count_exit_code": parse.returncode,
            "stdout": parse.stdout.strip(),
            "stderr": parse.stderr.strip(),
        }
    )
    OUT.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))
    return 0 if result["status"] in {"pass", "skip"} else 1


if __name__ == "__main__":
    raise SystemExit(main())
