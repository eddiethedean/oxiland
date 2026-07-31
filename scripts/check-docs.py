#!/usr/bin/env python3
"""Checks repository-local links in Markdown documentation."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parents[1]
LINK = re.compile(r"!?\[[^\]]*]\(([^)]+)\)")
EXCLUDED_DIRS = {".git", "target", "site", ".venv", ".pytest_cache", "node_modules"}


def markdown_files() -> list[Path]:
    return sorted(
        path
        for path in ROOT.rglob("*.md")
        if not any(part in EXCLUDED_DIRS for part in path.relative_to(ROOT).parts)
    )


def prose_lines(path: Path):
    in_fence = False
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if line.lstrip().startswith("```"):
            in_fence = not in_fence
            continue
        if not in_fence:
            yield number, line


def local_destination(raw: str) -> str | None:
    destination = raw.strip()
    if destination.startswith("<") and ">" in destination:
        destination = destination[1 : destination.index(">")]
    else:
        destination = destination.split(maxsplit=1)[0]

    if (
        not destination
        or destination.startswith("#")
        or "://" in destination
        or destination.startswith(("mailto:", "data:"))
    ):
        return None
    return unquote(destination.split("#", 1)[0])


def main() -> int:
    failures: list[str] = []
    checked = 0

    for document in markdown_files():
        for line_number, line in prose_lines(document):
            for match in LINK.finditer(line):
                destination = local_destination(match.group(1))
                if destination is None:
                    continue
                checked += 1
                target = (document.parent / destination).resolve()
                if ROOT != target and ROOT not in target.parents:
                    failures.append(
                        f"{document.relative_to(ROOT)}:{line_number}: "
                        f"link escapes repository: {destination}"
                    )
                elif not target.exists():
                    failures.append(
                        f"{document.relative_to(ROOT)}:{line_number}: "
                        f"missing link target: {destination}"
                    )

    if failures:
        print("\n".join(failures), file=sys.stderr)
        print(
            f"documentation link check failed: {len(failures)} error(s)",
            file=sys.stderr,
        )
        return 1

    print(
        f"documentation link check passed: "
        f"{len(markdown_files())} files, {checked} local links"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
