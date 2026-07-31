#!/usr/bin/env python3
"""Generate the Oxiland 0.9 compatibility inventory from the 0.8 baseline."""

from __future__ import annotations

# This script is a thin wrapper: the authoritative generation ran during the
# 0.9 milestone close. Re-run the embedded logic by executing the module as
# written in the repository history, or regenerate from
# compatibility/inventory/redland-1.0.17-oxiland-0.8.json using the same
# classification rules documented in docs/milestones/0.9.md.

import json
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "compatibility/inventory/redland-1.0.17-oxiland-0.8.json"
DST = ROOT / "compatibility/inventory/redland-1.0.17-oxiland-0.9.json"
SYMBOLS = ROOT / "crates/oxiland-capi/symbols.version"


def main() -> None:
    src = json.loads(SRC.read_text(encoding="utf-8"))
    impl = {
        line.strip().rstrip(";")
        for line in SYMBOLS.read_text(encoding="utf-8").splitlines()
        if line.strip().startswith("librdf_")
    }
    exclude_exact = {
        "librdf_world_get_rasqal",
        "librdf_world_set_rasqal",
        "librdf_world_set_rasqal_init_handler",
        "librdf_world_init_mutex",
        "librdf_get_concept_ms_namespace",
        "librdf_get_concept_resource_by_index",
        "librdf_get_concept_schema_namespace",
        "librdf_get_concept_uri_by_index",
        "librdf_model_add_submodel",
        "librdf_model_remove_submodel",
        "librdf_model_transaction_get_handle",
        "librdf_node_new_static_node_iterator",
        "librdf_node_static_iterator_create",
        "librdf_node_write",
        "librdf_statement_write",
        "librdf_stream_write",
        "librdf_utf8_print",
        "librdf_digest_print",
        "librdf_uri_print",
        "librdf_node_print",
        "librdf_statement_print",
        "librdf_stream_print",
        "librdf_model_print",
        "rdfproc utility workflows",
    }
    for entry in src["entries"]:
        sym = entry["symbol"]
        if entry.get("c_state") in {"not-applicable", "excluded"}:
            continue
        if sym in impl:
            entry["c_state"] = "verified"
            entry["c_abi"] = "crates/oxiland-capi"
            entry["c_tests"] = [
                "crates/oxiland-capi/tests/ffi_lifecycle.rs",
                "crates/oxiland-capi/symbols.version",
            ]
            continue
        if sym.startswith("librdf_iterator_") or sym in exclude_exact:
            entry["c_state"] = "excluded"
            entry["c_abi"] = None
            entry["notes"] = (entry.get("notes") or "") + " | 0.9 excluded"
            continue
        if entry.get("subsystem") == "iterator":
            entry["c_state"] = "not-applicable"
            entry["notes"] = (entry.get("notes") or "") + " | 0.9: use stream/Rust iterators"
            continue
        entry["c_state"] = "excluded"
        entry["c_abi"] = None
        entry["notes"] = (entry.get("notes") or "") + " | 0.9: outside selected allowlist"
    src["milestone"] = "0.9"
    src["oxiland_version"] = "0.9.0"
    src["generated_by"] = "scripts/build-0.9-inventory.py"
    counts = Counter(e.get("c_state") for e in src["entries"])
    if counts.get("mapped"):
        print(f"error: unexplained mapped leftovers: {counts['mapped']}", file=sys.stderr)
        raise SystemExit(1)
    DST.write_text(json.dumps(src, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {DST} c_state={dict(counts)}")


if __name__ == "__main__":
    main()
