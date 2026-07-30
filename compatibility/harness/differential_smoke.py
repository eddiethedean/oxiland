#!/usr/bin/env python3
"""Compare Oxiland and rapper counts for the I/O smoke fixture."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "compatibility" / "fixtures" / "io" / "smoke.ttl"
OUT = ROOT / "compatibility" / "harness" / "differential-smoke-result.json"


def oxiland_count() -> tuple[str, int | None, str]:
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        (tmp_path / "src").mkdir()
        manifest = (
            "[package]\n"
            'name = "oxiland_diff_smoke"\n'
            'version = "0.0.0"\n'
            'edition = "2024"\n'
            "publish = false\n"
            "\n"
            "[dependencies]\n"
            f"oxiland = {{ path = {json.dumps(str(ROOT))} }}\n"
        )
        (tmp_path / "Cargo.toml").write_text(manifest, encoding="utf-8")
        (tmp_path / "src" / "main.rs").write_text(
            f"""
use oxiland::io::{{Parser, Syntax}};
use std::fs;
fn main() {{
    let data = fs::read({json.dumps(str(FIXTURE))}).unwrap();
    let n = Parser::for_syntax(Syntax::Turtle)
        .parse_slice(&data)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .len();
    println!("{{n}}");
}}
""",
            encoding="utf-8",
        )
        built = subprocess.run(
            ["cargo", "run", "--quiet", "--manifest-path", str(tmp_path / "Cargo.toml")],
            check=False,
            capture_output=True,
            text=True,
        )
        if built.returncode != 0:
            return "fail", None, built.stderr
        return "pass", int(built.stdout.strip().splitlines()[-1]), ""


def main() -> int:
    ox_status, ox_count, ox_err = oxiland_count()
    rapper = shutil.which("rapper")
    result = {
        "schema_version": 1,
        "suite": "oxiland-io-differential-smoke",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "fixture": str(FIXTURE.relative_to(ROOT)),
        "oxiland": {"status": ox_status, "quads": ox_count, "stderr": ox_err},
    }
    if rapper is None:
        result["native"] = {
            "status": "skip",
            "reason": "rapper not installed",
        }
        result["status"] = "skip" if ox_status == "pass" else "fail"
    else:
        native = subprocess.run(
            [rapper, "-i", "turtle", "-c", str(FIXTURE)],
            check=False,
            capture_output=True,
            text=True,
        )
        # rapper -c prints "rapper: Parsing returned N triples"
        native_count = None
        for line in (native.stderr + "\n" + native.stdout).splitlines():
            parts = line.split()
            for idx, part in enumerate(parts):
                if part.isdigit() and idx + 1 < len(parts) and "triple" in parts[idx + 1]:
                    native_count = int(part)
        result["native"] = {
            "status": "pass" if native.returncode == 0 else "fail",
            "quads": native_count,
            "stderr": native.stderr.strip(),
        }
        if ox_status == "pass" and result["native"]["status"] == "pass":
            if ox_count == native_count:
                result["status"] = "pass"
                result["classification"] = "equal"
            else:
                result["status"] = "fail"
                result["classification"] = "count-mismatch"
        else:
            result["status"] = "fail"
            result["classification"] = "runner-failure"

    OUT.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))
    return 0 if result["status"] in {"pass", "skip"} else 1


if __name__ == "__main__":
    raise SystemExit(main())
