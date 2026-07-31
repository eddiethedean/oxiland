#!/usr/bin/env python3
"""Require a release tag to match every independently published package."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys
import tomllib


ROOT = Path(__file__).resolve().parents[1]
MANIFESTS = {
    "oxiland": ROOT / "Cargo.toml",
    "oxiland-cli": ROOT / "crates/oxiland-cli/Cargo.toml",
    "oxiland-py crate": ROOT / "python/Cargo.toml",
    "PyPI oxiland": ROOT / "python/pyproject.toml",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("tag", help="release tag, with or without a leading v")
    args = parser.parse_args()
    expected = args.tag.removeprefix("v")

    versions: dict[str, str] = {}
    for package, path in MANIFESTS.items():
        with path.open("rb") as source:
            versions[package] = tomllib.load(source)["package" if path.name == "Cargo.toml" else "project"]["version"]

    mismatches = {
        package: version for package, version in versions.items() if version != expected
    }
    if mismatches:
        print(f"release tag {args.tag!r} resolves to {expected!r}", file=sys.stderr)
        for package, version in versions.items():
            marker = "MISMATCH" if package in mismatches else "ok"
            print(f"  {marker:8} {package}: {version}", file=sys.stderr)
        return 1

    packages = ", ".join(versions)
    print(f"Release version {expected} matches: {packages}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
