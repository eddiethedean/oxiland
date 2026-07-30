#!/usr/bin/env python3
"""SPARQL smoke fixture for Oxiland 0.3.

Loads `compatibility/fixtures/sparql/smoke.ttl` through the curated
`sparql_fixture_smoke` integration test. Prefer rasqal when available for
differential expansion; until then classify honestly as oxiland-facade.
"""

from __future__ import annotations

import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "compatibility" / "fixtures" / "sparql" / "smoke.ttl"


def main() -> int:
    if not FIXTURE.is_file():
        print(f"missing SPARQL smoke fixture: {FIXTURE}", file=sys.stderr)
        return 1

    proc = subprocess.run(
        [
            "cargo",
            "test",
            "--test",
            "query",
            "sparql_fixture_smoke",
            "--",
            "--exact",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    status = "pass" if proc.returncode == 0 else "fail"
    payload = {
        "schema_version": 1,
        "suite": "oxiland-sparql-smoke",
        "fixture": str(FIXTURE.relative_to(ROOT)),
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "classification": "oxiland-facade",
        "notes": (
            "Runs tests/query.rs::sparql_fixture_smoke against smoke.ttl. "
            "Rasqal differential expands when oracle fixtures land."
        ),
        "cargo_exit_code": proc.returncode,
        "stderr_tail": proc.stderr[-500:],
    }
    out = ROOT / "compatibility" / "harness" / "sparql-smoke-result.json"
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(payload, indent=2))
    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
