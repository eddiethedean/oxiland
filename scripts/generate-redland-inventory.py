#!/usr/bin/env python3
"""Generate header-derived Redland → Oxiland 0.6 inventory classifications.

Reads public Redland 1.0.17 headers (downloaded tarball or REDLAND_SRC),
extracts librdf_* function symbols, classifies them, and writes
compatibility/inventory/redland-1.0.17-oxiland-0.6.json.

Classifications are rule-based (ADR-018–ADR-021). Re-run after reviewing
diffs; do not silently accept accidental mass changes in PRs without review.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "compatibility" / "baseline"
SHA_FILE = BASELINE / "redland-1.0.17.sha256"
OUT = ROOT / "compatibility" / "inventory" / "redland-1.0.17-oxiland-0.6.json"
TARBALL_URL = "https://download.librdf.org/source/redland-1.0.17.tar.gz"
EXPECTED_SHA = SHA_FILE.read_text(encoding="utf-8").split()[0]

ACCOUNTING_DOC = "docs/design/0.6-safe-api-accounting.md"
ACCOUNTING_TEST = "tests/accounting.rs::inventory_accounting_families"
CLI_DESIGN = "docs/design/0.6-cli-rdfproc.md"

FN_RE = re.compile(
    r"(?:^|\n)\s*(?:REDEFINING_[A-Z_]+\s+)?"
    r"(?:[A-Za-z_][A-Za-z0-9_\s\*]*?)\b(librdf_[a-zA-Z0-9_]+)\s*\("
)


def fail(msg: str) -> None:
    print(f"error: {msg}", file=sys.stderr)
    raise SystemExit(1)


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def ensure_headers(src_override: Path | None) -> Path:
    if src_override is not None:
        src = src_override / "src" if (src_override / "src").is_dir() else src_override
        if not (src / "librdf.h").is_file():
            fail(f"librdf.h not found under {src_override}")
        return src

    cache = Path(tempfile.gettempdir()) / "oxiland-redland-1.0.17"
    src = cache / "src"
    if (src / "librdf.h").is_file():
        return src

    tarball = cache.with_suffix(".tar.gz")
    cache.mkdir(parents=True, exist_ok=True)
    if not tarball.is_file():
        print(f"fetching {TARBALL_URL} …", file=sys.stderr)
        urllib.request.urlretrieve(TARBALL_URL, tarball)
    digest = sha256_file(tarball)
    if digest != EXPECTED_SHA:
        fail(f"tarball sha256 mismatch: got {digest}, expected {EXPECTED_SHA}")
    with tarfile.open(tarball, "r:gz") as tf:
        tf.extractall(cache / "_extract")
    extracted = next((cache / "_extract").iterdir())
    if src.exists():
        shutil.rmtree(src)
    shutil.move(str(extracted / "src"), str(src))
    return src


def public_headers(src: Path) -> list[Path]:
    headers = []
    for path in sorted(src.glob("*.h")):
        name = path.name
        if name.endswith("_internal.h"):
            continue
        if name in {"win32_rdf_config.h", "rdf_internal.h"}:
            continue
        headers.append(path)
    return headers


def extract_functions(src: Path) -> list[tuple[str, str]]:
    found: list[tuple[str, str]] = []
    seen: set[str] = set()
    for header in public_headers(src):
        text = header.read_text(encoding="utf-8", errors="ignore")
        for match in FN_RE.finditer(text):
            name = match.group(1)
            if name.endswith("_H") or name in seen:
                continue
            seen.add(name)
            found.append((name, header.name))
    return found


def subsystem_for(name: str, header: str) -> str:
    if header.startswith("rdf_") and header.endswith(".h"):
        return header[len("rdf_") : -len(".h")].replace("_module", "")
    parts = name.split("_")
    return parts[1] if len(parts) > 1 else "misc"


def classify(name: str, header: str) -> dict:
    """Return state, safe_rust, implementation, tests, optional notes/deviations."""
    sub = subsystem_for(name, header)
    lower = name.lower()

    def na(reason: str, safe: str) -> dict:
        return {
            "state": "not-applicable",
            "safe_rust": safe,
            "implementation": ACCOUNTING_DOC,
            "tests": [ACCOUNTING_TEST],
            "notes": reason,
        }

    def excluded(reason: str, workaround: str) -> dict:
        return {
            "state": "excluded",
            "safe_rust": "none (excluded)",
            "implementation": ACCOUNTING_DOC,
            "tests": [ACCOUNTING_TEST],
            "notes": reason,
            "deviations": [
                {
                    "impact": reason,
                    "workaround": workaround,
                    "owner": "oxiland maintainers",
                    "review_date": "2026-07-30",
                }
            ],
        }

    def verified(safe: str, impl: str, tests: list[str]) -> dict:
        return {
            "state": "verified",
            "safe_rust": safe,
            "implementation": impl,
            "tests": tests,
        }

    # Ownership constructors/destructors → Rust RAII
    if re.match(r"librdf_free_", lower) or re.match(r"librdf_new_", lower):
        if "hash" in lower or sub == "hash":
            return na("Use HashMap; ADR-016", "std::collections::HashMap")
        if "list" in lower or sub == "list":
            return na("Use Vec/iterators; ADR-016", "Vec / iterators")
        if "digest" in lower:
            return verified(
                "utility::DigestAlgorithm / digest_*",
                "src/utility/digest.rs",
                ["tests/utility.rs::digest_hex_known_vectors"],
            )
        return na("Rust ownership / Drop replaces new/free pairs", "RAII types")

    if sub in {"hash"} or "librdf_hash" in lower:
        return na("ADR-016: use HashMap", "std::collections::HashMap")
    if sub in {"list"} or "librdf_list" in lower:
        return na("ADR-016: use Vec", "Vec / iterators")

    if "register" in lower or "unregister" in lower or sub == "storage_module":
        return excluded(
            "Custom factory/plugin registration is out of safe-Rust scope (ADR-018)",
            "Use closed Syntax / ResultsFormat / StorageBackend discovery",
        )

    if sub == "raptor" or "raptor" in lower:
        return excluded(
            "Raptor embedding/world bridge is C-integration only",
            "Use oxiland::io Parser/Serializer over Oxigraph",
        )

    if sub in {"types"} or lower in {
        "librdf_alloc_memory",
        "librdf_calloc_memory",
        "librdf_free_memory",
    }:
        return na("Rust allocator / Box", "standard Rust allocation")

    # Per-object feature APIs (parser/serializer/model) are not on the facade.
    if "_feature" in lower or lower.endswith("feature") or "get_feature" in lower or "set_feature" in lower:
        if sub in {"model", "parser", "serializer", "storage", "query"}:
            return excluded(
                f"{name} feature get/set is not exposed on the Oxiland facade",
                "Use World::feature / set_feature for process features; Syntax/ResultsFormat for formats",
            )

    # Digests
    if sub == "digest" or "digest" in lower:
        return verified(
            "utility::DigestAlgorithm / digest_*",
            "src/utility/digest.rs",
            ["tests/utility.rs::digest_hex_known_vectors"],
        )

    # URI / files / utf8 / heuristics
    if sub == "uri" or lower.startswith("librdf_uri"):
        return verified(
            "utility::join_iri / resolve_iri / path_to_file_uri / …",
            "src/utility/uri.rs",
            ["tests/utility.rs::uri_join_and_relativize"],
        )
    if sub == "files" or "temporary_file" in lower:
        return na(
            "Use std::env::temp_dir / the tempfile crate; no librdf files helper",
            "std::env::temp_dir",
        )
    if sub in {"utf8", "latin1"} or "utf8" in lower or "unicode" in lower:
        return verified(
            "utility::normalize_nfc / normalize_nfkc",
            "src/utility/unicode.rs",
            ["tests/utility.rs::unicode_normalization_helpers"],
        )
    if sub == "heuristics" or "heuristic" in lower:
        return na(
            "CLI uses simple IRI/blank/literal heuristics; no librdf_heuristic API",
            "oxiland-cli node parsing + utility::uri",
        )

    # Logging / world / init
    if sub in {"log", "init"} or lower.startswith("librdf_world") or lower.startswith(
        "librdf_log"
    ):
        return verified(
            "World / LogLevel / LogFacility / set_log_handler",
            "src/world.rs",
            [
                "tests/utility.rs::logging_filters_and_preserves_order",
                "tests/model.rs::world_features_are_shared_across_clones",
            ],
        )

    # Concepts / vocab
    if sub == "concepts" or "concept" in lower:
        return verified(
            "utility::vocab::{rdf,rdfs,xsd,owl,dc}",
            "src/utility/vocab/mod.rs",
            ["tests/utility.rs::namespace_expand_and_vocab_constants"],
        )

    # Model / statement / node / stream / iterator
    if sub in {"model", "statement", "node", "stream", "iterator"}:
        if sub == "stream" or sub == "iterator":
            return verified(
                "StatementMatches / QuadStream / QueryResults (ADR-013)",
                "docs/design/0.5-streams-utilities.md",
                ["tests/utility.rs::find_early_stop_evidence_matrix"],
            )
        if sub == "node":
            return verified(
                "terms::{NamedNode,BlankNode,Literal,…}",
                "src/lib.rs",
                ["tests/model.rs::model_supports_redland_style_crud_and_matching"],
            )
        if sub == "statement":
            return verified(
                "terms::{Triple,Quad} / StatementPattern",
                "src/model.rs",
                ["tests/model.rs::model_supports_redland_style_crud_and_matching"],
            )
        # model
        if any(x in lower for x in ("storage", "sync", "transaction", "serializ")):
            return verified(
                "Model::open / transaction / sync / Serializer",
                "src/model.rs",
                ["tests/storage.rs::transaction_commit_persists_fjall"],
            )
        return verified(
            "Model CRUD / find / contexts",
            "src/model.rs",
            ["tests/model.rs::model_supports_redland_style_crud_and_matching"],
        )

    # Parser / serializer
    if sub == "parser":
        return verified(
            "io::Parser / Syntax / GraphTarget",
            "src/io/parser.rs",
            ["tests/io.rs::parser_streams_and_supports_early_stop"],
        )
    if sub == "serializer":
        return verified(
            "io::Serializer / Syntax",
            "src/io/serializer.rs",
            ["tests/io.rs::round_trip_each_advertised_syntax"],
        )

    # Query
    if sub == "query":
        return verified(
            "Query / Update / QueryResults / ResultsFormat",
            "src/query.rs",
            ["tests/query.rs::ask_select_construct_describe_positive_paths"],
        )

    # Storage
    if sub == "storage":
        if any(
            x in lower
            for x in (
                "mysql",
                "virtuoso",
                "postgresql",
                "sqlite",
                "tstore",
                "hashes",
            )
        ):
            return excluded(
                "Third-party / legacy Redland storage plugins are not part of Oxiland",
                "Use Model::new (memory) or Model::open (Fjall format v1)",
            )
        if "module" in lower or "register" in lower:
            return excluded(
                "Storage module registration excluded (ADR-018)",
                "Use StorageBackend::Memory / Fjall",
            )
        if any(
            x in lower
            for x in (
                "open",
                "new_storage",
                "free_storage",
                "size",
                "supports_query",
                "enumerate",
                "get_description",
                "sync",
            )
        ) or lower in {
            "librdf_storage_size",
            "librdf_free_storage",
        }:
            return verified(
                "Model::new / Model::open / OpenOptions / StorageCapabilities",
                "src/storage/mod.rs",
                ["tests/storage.rs::capabilities_memory_vs_fjall"],
            )
        return excluded(
            "Per-storage Redland methods are not mirrored 1:1 on DiskStore",
            "Use Model CRUD, find, Query/Update, and OpenOptions",
        )

    # Fallback
    return excluded(
        f"No idiomatic safe-Rust mapping selected for {name} in 0.6",
        "See migration guide; use facade workflows or file an issue",
    )


def make_id(name: str) -> str:
    return "librdf." + name.removeprefix("librdf_").replace("_", ".")


def kind_for(name: str) -> str:
    if name.startswith("librdf_new_") or name.startswith("librdf_free_"):
        return "function"
    return "function"


def build_entries(functions: list[tuple[str, str]]) -> list[dict]:
    entries = []
    for name, header in functions:
        meta = classify(name, header)
        entry = {
            "id": make_id(name),
            "symbol": name,
            "kind": kind_for(name),
            "subsystem": subsystem_for(name, header),
            "header": header,
            "safe_rust": meta["safe_rust"],
            "implementation": meta["implementation"],
            "tests": meta["tests"],
            "state": meta["state"],
        }
        if "notes" in meta:
            entry["notes"] = meta["notes"]
        if "deviations" in meta:
            entry["deviations"] = meta["deviations"]
        entries.append(entry)
    # Stable sort
    entries.sort(key=lambda e: e["id"])
    return entries


def add_cli_row(entries: list[dict]) -> None:
    entries.append(
        {
            "id": "librdf.rdfproc.cli",
            "symbol": "rdfproc utility workflows",
            "kind": "utility",
            "subsystem": "rdfproc",
            "header": "rdfproc(1)",
            "safe_rust": "crates/oxiland-cli (ADR-019)",
            "implementation": CLI_DESIGN,
            "tests": ["crates/oxiland-cli/tests/cli_workflows.rs::parse_find_query_round_trip"],
            "state": "verified",
            "notes": "Workflow-compatible CLI; not a binary drop-in for native rdfproc.",
        }
    )
    entries.sort(key=lambda e: e["id"])


def build_document(src: Path) -> dict:
    functions = extract_functions(src)
    if len(functions) < 100:
        fail(f"expected hundreds of librdf functions, found {len(functions)}")
    entries = build_entries(functions)
    add_cli_row(entries)
    return {
        "schema_version": 1,
        "milestone": "0.6",
        "redland_api": "1.0.17",
        "redland_manual": "1.0.18",
        "oxiland_version": "0.6.0",
        "oxigraph_version": "0.5.9",
        "generated_by": "scripts/generate-redland-inventory.py",
        "redland_tarball_sha256": EXPECTED_SHA,
        "notes": (
            "Header-derived full public librdf function inventory for safe-API "
            "accounting (ADR-018–ADR-021). States are classified by subsystem "
            "rules; excluded rows carry deviation metadata."
        ),
        "entries": entries,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--redland-src",
        type=Path,
        help="Path to extracted redland-1.0.17 tree or its src/ directory",
    )
    parser.add_argument(
        "--check-only",
        action="store_true",
        help="Fail if regenerating would change the checked-in 0.6 inventory",
    )
    args = parser.parse_args()

    src = ensure_headers(args.redland_src)
    doc = build_document(src)
    rendered = json.dumps(doc, indent=2) + "\n"

    if args.check_only:
        if not OUT.is_file():
            fail(f"missing {OUT}")
        current = OUT.read_text(encoding="utf-8")
        if current != rendered:
            fail(
                f"{OUT.relative_to(ROOT)} is out of date with generator rules; "
                "re-run scripts/generate-redland-inventory.py and review the diff"
            )
        print(f"ok: {OUT.relative_to(ROOT)} matches generator output")
        return

    OUT.write_text(rendered, encoding="utf-8")
    counts: dict[str, int] = defaultdict(int)
    for e in doc["entries"]:
        counts[e["state"]] += 1
    print(f"wrote {OUT.relative_to(ROOT)} with {len(doc['entries'])} entries")
    for state in sorted(counts):
        print(f"  {state}: {counts[state]}")


if __name__ == "__main__":
    main()
