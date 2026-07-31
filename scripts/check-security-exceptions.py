#!/usr/bin/env python3
"""Keep temporary RustSec exceptions narrow and self-expiring."""

from __future__ import annotations

from pathlib import Path
import tomllib


ROOT = Path(__file__).resolve().parents[1]
EXPECTED_OXIGRAPH = "=0.5.9"
EXPECTED_QUICK_XML = "0.37.5"


def package_versions(lockfile: Path, name: str) -> set[str]:
    with lockfile.open("rb") as source:
        packages = tomllib.load(source)["package"]
    return {package["version"] for package in packages if package["name"] == name}


def dependency_version(manifest: Path, name: str) -> str:
    with manifest.open("rb") as source:
        dependency = tomllib.load(source)["dependencies"][name]
    return dependency["version"] if isinstance(dependency, dict) else dependency


def main() -> int:
    manifests = (ROOT / "Cargo.toml", ROOT / "python/Cargo.toml")
    lockfiles = (ROOT / "Cargo.lock", ROOT / "python/Cargo.lock")

    for manifest in manifests:
        actual = dependency_version(manifest, "oxigraph")
        if actual != EXPECTED_OXIGRAPH:
            raise SystemExit(
                f"{manifest}: Oxigraph changed from {EXPECTED_OXIGRAPH}; "
                "remove or re-review the quick-xml audit exceptions"
            )

    for lockfile in lockfiles:
        actual = package_versions(lockfile, "quick-xml")
        if actual != {EXPECTED_QUICK_XML}:
            raise SystemExit(
                f"{lockfile}: quick-xml changed from {EXPECTED_QUICK_XML}; "
                "remove or re-review RUSTSEC-2026-0194/0195 exceptions"
            )

    pyo3 = package_versions(ROOT / "python/Cargo.lock", "pyo3")
    if len(pyo3) != 1 or tuple(map(int, next(iter(pyo3)).split("."))) < (0, 29, 0):
        raise SystemExit(f"unexpected PyO3 lock version: {sorted(pyo3)}")

    print(
        "Verified narrow R-020 exception: Oxigraph 0.5.9 -> quick-xml 0.37.5; "
        "PyO3 is patched at 0.29.0"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
