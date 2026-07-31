#!/usr/bin/env python3
"""Checks repository-local links in Markdown and embedded HTML."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parents[1]
LINK = re.compile(r"!?\[[^\]]*]\(([^)]+)\)")
HTML_LINK = re.compile(r'''\b(?:href|src)\s*=\s*["']([^"']+)["']''', re.I)
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
    destination = destination.split("#", 1)[0].split("?", 1)[0]
    return unquote(destination)


def source_target(document: Path, destination: str) -> Path:
    """Resolves both source-file links and MkDocs directory-style URLs."""
    target = (document.parent / destination).resolve()
    if target.exists():
        return target

    route = target if not destination.endswith("/") else target.parent / target.name
    markdown_target = route.with_suffix(".md")
    if markdown_target.exists():
        return markdown_target

    index_target = route / "index.md"
    if index_target.exists():
        return index_target
    return target


def main() -> int:
    failures: list[str] = []
    checked = 0

    for document in markdown_files():
        for line_number, line in prose_lines(document):
            raw_destinations = [match.group(1) for match in LINK.finditer(line)]
            raw_destinations.extend(
                match.group(1) for match in HTML_LINK.finditer(line)
            )
            for raw_destination in raw_destinations:
                destination = local_destination(raw_destination)
                if destination is None:
                    continue
                checked += 1
                target = source_target(document, destination)
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
