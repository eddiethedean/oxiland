#!/usr/bin/env python3
"""Validate the complete metadata and contents of Oxiland release wheels."""

from __future__ import annotations

import argparse
from collections import Counter
from email.parser import BytesParser
from pathlib import Path
import re
import sys
import tomllib
from zipfile import BadZipFile, ZipFile


SUPPORTED_ABIS = ("cp310", "cp311", "cp312", "cp313", "cp314")
PROJECT_URLS = {"Changelog", "Documentation", "Homepage", "Issues", "Repository"}
ROOT = Path(__file__).resolve().parents[1]


def fail(message: str) -> None:
    raise ValueError(message)


def one(names: list[str], suffix: str, wheel: Path) -> str:
    matches = [name for name in names if name.endswith(suffix)]
    if len(matches) != 1:
        fail(f"{wheel.name}: expected one {suffix!r}, found {matches}")
    return matches[0]


def validate_wheel(wheel: Path, version: str) -> str:
    filename = re.fullmatch(
        rf"oxiland-{re.escape(version)}-(cp3(?:10|11|12|13|14))-\1-.+\.whl",
        wheel.name,
    )
    if filename is None:
        fail(f"unexpected wheel filename: {wheel.name}")
    abi = filename.group(1)

    try:
        with ZipFile(wheel) as archive:
            bad_member = archive.testzip()
            if bad_member is not None:
                fail(f"{wheel.name}: corrupt member {bad_member}")
            names = archive.namelist()
            if len(names) != len(set(names)):
                fail(f"{wheel.name}: duplicate archive members")

            required = {
                "oxiland/__init__.py",
                "oxiland/__init__.pyi",
                "oxiland/py.typed",
            }
            missing = sorted(required.difference(names))
            if missing:
                fail(f"{wheel.name}: missing package files {missing}")
            if "py.typed" in names:
                fail(f"{wheel.name}: py.typed must be inside the oxiland package")

            native = [
                name
                for name in names
                if name.startswith("oxiland/oxiland.")
                and name.endswith((".so", ".pyd", ".dylib"))
            ]
            if len(native) != 1:
                fail(f"{wheel.name}: expected one native extension, found {native}")

            metadata_name = one(names, ".dist-info/METADATA", wheel)
            wheel_name = one(names, ".dist-info/WHEEL", wheel)
            one(names, ".dist-info/RECORD", wheel)
            one(names, ".dist-info/licenses/LICENSE-APACHE", wheel)
            one(names, ".dist-info/licenses/LICENSE-MIT", wheel)
            sbom_name = one(names, ".dist-info/sboms/oxiland-py.cyclonedx.json", wheel)

            metadata = BytesParser().parsebytes(archive.read(metadata_name))
            expected_fields = {
                "Name": "oxiland",
                "Version": version,
                "Requires-Python": ">=3.10",
                "License": "Apache-2.0 OR MIT",
            }
            for field, expected in expected_fields.items():
                actual = metadata[field]
                if actual != expected:
                    fail(
                        f"{wheel.name}: {field} must be {expected!r}, got {actual!r}"
                    )
            urls = {
                value.split(",", 1)[0]
                for value in metadata.get_all("Project-URL", [])
                if "," in value
            }
            if urls != PROJECT_URLS:
                fail(f"{wheel.name}: unexpected project URL labels {sorted(urls)}")
            if "Oxiland is a typed Python library" not in metadata.get_payload():
                fail(f"{wheel.name}: production package README is missing")

            wheel_metadata = BytesParser().parsebytes(archive.read(wheel_name))
            if wheel_metadata["Root-Is-Purelib"] != "false":
                fail(f"{wheel.name}: native wheel must not be pure Python")
            tags = wheel_metadata.get_all("Tag", [])
            if not any(tag.startswith(f"{abi}-{abi}-") for tag in tags):
                fail(f"{wheel.name}: WHEEL tags do not match filename ABI: {tags}")

            sbom = archive.read(sbom_name)
            if b'"bomFormat": "CycloneDX"' not in sbom:
                fail(f"{wheel.name}: malformed or missing CycloneDX SBOM")
    except BadZipFile as error:
        fail(f"{wheel.name}: invalid zip archive: {error}")

    return abi


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("directory", type=Path)
    parser.add_argument("--version")
    parser.add_argument("--copies-per-abi", required=True, type=int)
    args = parser.parse_args()

    if args.version is None:
        with (ROOT / "python/pyproject.toml").open("rb") as source:
            args.version = tomllib.load(source)["project"]["version"]

    wheels = sorted(args.directory.glob("oxiland-*.whl"))
    expected_count = len(SUPPORTED_ABIS) * args.copies_per_abi
    if len(wheels) != expected_count:
        fail(f"expected {expected_count} wheels in {args.directory}, found {wheels}")

    counts = Counter(validate_wheel(wheel, args.version) for wheel in wheels)
    expected = {abi: args.copies_per_abi for abi in SUPPORTED_ABIS}
    if counts != expected:
        fail(f"expected ABI distribution {expected}, found {dict(counts)}")

    print(f"Validated {len(wheels)} Oxiland {args.version} wheels: {dict(counts)}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"wheel validation failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
