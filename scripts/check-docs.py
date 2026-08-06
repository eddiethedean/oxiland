#!/usr/bin/env python3
"""Checks repository-local links and adoption-critical doc consistency."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parents[1]
LINK = re.compile(r"!?\[[^\]]*]\(([^)]+)\)")
HTML_LINK = re.compile(r'''\b(?:href|src)\s*=\s*["']([^"']+)["']''', re.I)
PACKAGE_VERSION = re.compile(
    r'^version\s*=\s*"(?P<version>\d+\.\d+\.\d+)"\s*$', re.M
)
EXCLUDED_DIRS = {".git", "target", "site", ".venv", ".pytest_cache", "node_modules"}

# Adoption-facing docs that must track the published package version.
VERSION_GUARD_PATHS = [
    ROOT / "README.md",
    ROOT / "SUPPORT.md",
    ROOT / "SECURITY.md",
    ROOT / "docs" / "index.md",
    *(ROOT / "docs" / "users").glob("*.md"),
]


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


def package_version() -> str:
    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    match = PACKAGE_VERSION.search(cargo)
    if match is None:
        raise SystemExit("Cargo.toml is missing a package version field")
    return match.group("version")


def version_guard_failures(current: str) -> list[str]:
    major, minor, _patch = current.split(".")
    expected_reopen = f"0.4.x–{major}.{minor}.x"
    stale_reopens = [
        "0.4.x–0.8.x",
        "0.4.x–0.10.x",
        "0.4.x–0.11.x",
        "0.4.x–0.12.x",
    ]
    # Drop the expected current reopen from the stale list if it coincides.
    stale_reopens = [item for item in stale_reopens if item != expected_reopen]

    previous_minor = str(int(minor) - 1) if int(minor) > 0 else None
    stale_pin = f"{major}.{previous_minor}.0" if previous_minor is not None else None

    failures: list[str] = []
    for path in VERSION_GUARD_PATHS:
        if not path.exists():
            failures.append(f"missing version-guard document: {path.relative_to(ROOT)}")
            continue
        text = path.read_text(encoding="utf-8")
        rel = path.relative_to(ROOT)

        if path.name == "upgrading.md":
            # Historical pins are expected on the upgrade page.
            continue

        if stale_pin is not None:
            for pattern in (
                f'oxiland = "{stale_pin}"',
                f"oxiland=={stale_pin}",
                f"Tip **{stale_pin}** is the current package version",
                f"version `{stale_pin}`",
            ):
                if pattern in text:
                    failures.append(
                        f"{rel}: stale package pin {pattern!r}; "
                        f"expected current {current}"
                    )

        for reopen in stale_reopens:
            if reopen in text:
                failures.append(
                    f"{rel}: stale format-v1 reopen {reopen!r}; "
                    f"expected {expected_reopen}"
                )

        if path.name == "SUPPORT.md":
            if f"**{major}.{minor}.x**" not in text or "Current" not in text:
                failures.append(
                    f"{rel}: supported release lines must mark "
                    f"{major}.{minor}.x as Current"
                )
            if "**0.9.x** | Current" in text:
                failures.append(f"{rel}: support table still marks 0.9.x Current")

        if path.name == "SECURITY.md":
            if f"{major}.{minor}.x" not in text:
                failures.append(
                    f"{rel}: supported versions table must include {major}.{minor}.x"
                )

        if path.name == "faq.md" and "Budgeted benchmark suites are not published" in text:
            failures.append(
                f"{rel}: FAQ still claims budgeted benchmark suites are unpublished"
            )

    return failures


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

    current = package_version()
    failures.extend(version_guard_failures(current))

    if failures:
        print("\n".join(failures), file=sys.stderr)
        print(
            f"documentation check failed: {len(failures)} error(s)",
            file=sys.stderr,
        )
        return 1

    print(
        f"documentation check passed: "
        f"{len(markdown_files())} files, {checked} local links, "
        f"version guard for {current}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
