#!/usr/bin/env python3
"""Oxiland-side differential fixtures for 0.9 (Redland optional)."""

from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = Path(__file__).resolve().parent / "fixtures.json"


def main() -> None:
    fixtures = json.loads(FIXTURES.read_text(encoding="utf-8"))
    # Behavioral check via the Rust example binary / C example is authoritative
    # in CI; this script records expected ASK outcomes for documentation.
    for fixture in fixtures:
        assert fixture["ask_expected"] in {True, False}
        print(f"fixture {fixture['id']}: ask_expected={fixture['ask_expected']}")
    print("differential fixtures recorded (oxiland harness)")


if __name__ == "__main__":
    main()
