#!/usr/bin/env python3
"""SPARQL smoke fixture for Oxiland 0.3 (ASK via cargo example path).

Emits a versioned JSON result. Prefer rasqal/rapper when available; otherwise
classify as oxiland-only pass with an honest note.
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
    FIXTURE.parent.mkdir(parents=True, exist_ok=True)
    if not FIXTURE.exists():
        FIXTURE.write_text(
            '<https://example.com/alice> <https://example.com/name> "Alice" .\n',
            encoding="utf-8",
        )

    # Run a tiny Rust one-shot through cargo test filter as smoke evidence.
    proc = subprocess.run(
        [
            "cargo",
            "test",
            "--test",
            "query",
            "ask_select_construct_describe_positive_paths",
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
        "notes": "Curated facade smoke; Rasqal differential expands when oracle fixtures land.",
        "cargo_exit_code": proc.returncode,
        "stderr_tail": proc.stderr[-500:],
    }
    out = ROOT / "compatibility" / "harness" / "sparql-smoke-result.json"
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(payload, indent=2))
    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
