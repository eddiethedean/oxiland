#!/usr/bin/env python3
"""Build the 0.8 inventory by enriching the 0.6 safe-API inventory with C ABI fields."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "compatibility" / "inventory" / "redland-1.0.17-oxiland-0.6.json"
OUT = ROOT / "compatibility" / "inventory" / "redland-1.0.17-oxiland-0.8.json"

# Milestone 0.8 preview allowlist → C implementation / tests.
PREVIEW: dict[str, dict[str, object]] = {
    "librdf_new_world": {
        "c_abi": "librdf_new_world",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::world_new_free_and_null_free"],
    },
    "librdf_free_world": {
        "c_abi": "librdf_free_world",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::world_new_free_and_null_free"],
    },
    "librdf_world_open": {
        "c_abi": "librdf_world_open",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::model_add_contains_size"],
    },
    "librdf_new_storage": {
        "c_abi": "librdf_new_storage",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::model_add_contains_size"],
    },
    "librdf_free_storage": {
        "c_abi": "librdf_free_storage",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::null_frees_are_noop"],
    },
    "librdf_storage_open": {
        "c_abi": "librdf_storage_open",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::model_add_contains_size"],
    },
    "librdf_new_model": {
        "c_abi": "librdf_new_model",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::model_add_contains_size"],
    },
    "librdf_free_model": {
        "c_abi": "librdf_free_model",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::model_add_contains_size"],
    },
    "librdf_model_add_statement": {
        "c_abi": "librdf_model_add_statement",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::model_add_contains_size"],
    },
    "librdf_model_remove_statement": {
        "c_abi": "librdf_model_remove_statement",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::model_add_contains_size"],
    },
    "librdf_model_contains_statement": {
        "c_abi": "librdf_model_contains_statement",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::model_add_contains_size"],
    },
    "librdf_model_size": {
        "c_abi": "librdf_model_size",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::model_add_contains_size"],
    },
    "librdf_model_find_statements": {
        "c_abi": "librdf_model_find_statements",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::find_stream_lifecycle"],
    },
    "librdf_new_uri": {
        "c_abi": "librdf_new_uri",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::invalid_utf8_rejected"],
    },
    "librdf_free_uri": {
        "c_abi": "librdf_free_uri",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::null_frees_are_noop"],
    },
    "librdf_new_node_from_uri_string": {
        "c_abi": "librdf_new_node_from_uri_string",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::model_add_contains_size"],
    },
    "librdf_new_node_from_literal": {
        "c_abi": "librdf_new_node_from_literal",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::model_add_contains_size"],
    },
    "librdf_free_node": {
        "c_abi": "librdf_free_node",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::double_free_is_defended"],
    },
    "librdf_new_statement_from_nodes": {
        "c_abi": "librdf_new_statement_from_nodes",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::model_add_contains_size"],
    },
    "librdf_free_statement": {
        "c_abi": "librdf_free_statement",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::null_frees_are_noop"],
    },
    "librdf_stream_end": {
        "c_abi": "librdf_stream_end",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::find_stream_lifecycle"],
    },
    "librdf_stream_next": {
        "c_abi": "librdf_stream_next",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::find_stream_lifecycle"],
    },
    "librdf_stream_get_object": {
        "c_abi": "librdf_stream_get_object",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::find_stream_lifecycle"],
    },
    "librdf_free_stream": {
        "c_abi": "librdf_free_stream",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::find_stream_lifecycle"],
    },
    "librdf_new_parser": {
        "c_abi": "librdf_new_parser",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::parse_turtle_and_ask_query"],
    },
    "librdf_free_parser": {
        "c_abi": "librdf_free_parser",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::parse_turtle_and_ask_query"],
    },
    "librdf_parser_check_name": {
        "c_abi": "librdf_parser_check_name",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::parse_turtle_and_ask_query"],
    },
    "librdf_parser_parse_string_into_model": {
        "c_abi": "librdf_parser_parse_string_into_model",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::parse_turtle_and_ask_query"],
    },
    "librdf_new_serializer": {
        "c_abi": "librdf_new_serializer",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::serialize_and_select"],
    },
    "librdf_free_serializer": {
        "c_abi": "librdf_free_serializer",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::serialize_and_select"],
    },
    "librdf_serializer_check_name": {
        "c_abi": "librdf_serializer_check_name",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::serialize_and_select"],
    },
    "librdf_serializer_serialize_model_to_string": {
        "c_abi": "librdf_serializer_serialize_model_to_string",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::serialize_and_select"],
    },
    "librdf_new_query": {
        "c_abi": "librdf_new_query",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::parse_turtle_and_ask_query"],
    },
    "librdf_free_query": {
        "c_abi": "librdf_free_query",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::parse_turtle_and_ask_query"],
    },
    "librdf_model_query_execute": {
        "c_abi": "librdf_model_query_execute",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::parse_turtle_and_ask_query"],
    },
    "librdf_query_results_is_boolean": {
        "c_abi": "librdf_query_results_is_boolean",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::parse_turtle_and_ask_query"],
    },
    "librdf_query_results_get_boolean": {
        "c_abi": "librdf_query_results_get_boolean",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::parse_turtle_and_ask_query"],
    },
    "librdf_query_results_is_bindings": {
        "c_abi": "librdf_query_results_is_bindings",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::serialize_and_select"],
    },
    "librdf_query_results_finished": {
        "c_abi": "librdf_query_results_finished",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::serialize_and_select"],
    },
    "librdf_query_results_next": {
        "c_abi": "librdf_query_results_next",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::serialize_and_select"],
    },
    "librdf_query_results_get_binding_name": {
        "c_abi": "librdf_query_results_get_binding_name",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::serialize_and_select"],
    },
    "librdf_query_results_get_binding_value": {
        "c_abi": "librdf_query_results_get_binding_value",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::serialize_and_select"],
    },
    "librdf_query_results_get_bindings_count": {
        "c_abi": "librdf_query_results_get_bindings_count",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::serialize_and_select"],
    },
    "librdf_free_query_results": {
        "c_abi": "librdf_free_query_results",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::serialize_and_select"],
    },
    "librdf_free_memory": {
        "c_abi": "librdf_free_memory",
        "c_state": "verified",
        "c_tests": ["crates/oxiland-capi/tests/ffi_lifecycle.rs::serialize_and_select"],
    },
}


def main() -> None:
    data = json.loads(SRC.read_text(encoding="utf-8"))
    data["milestone"] = "0.8"
    data["oxiland_version"] = "0.8.0"
    data["generated_by"] = "scripts/build-0.8-inventory.py"
    data["notes"] = (
        "0.8 inventory extends 0.6 safe-API accounting with C ABI preview fields "
        "(ADR-022/023). Preview allowlist symbols are c_state=verified; remaining "
        "symbols are mapped (deferred to 0.9) unless already not-applicable/excluded."
    )
    for entry in data["entries"]:
        symbol = entry["symbol"]
        if symbol in PREVIEW:
            entry.update(PREVIEW[symbol])
            if entry.get("implementation", "").startswith("docs/") or entry[
                "implementation"
            ].startswith("src/"):
                # Point C-verified rows at the CAPI crate when useful.
                if symbol.startswith("librdf_"):
                    entry.setdefault("notes", "")
        else:
            state = entry.get("state")
            if state in {"not-applicable", "excluded"}:
                entry["c_abi"] = None
                entry["c_state"] = state
                entry["notes"] = (
                    (entry.get("notes") or "")
                    + ("; " if entry.get("notes") else "")
                    + "C ABI follows safe-API disposition for 0.8 preview"
                ).strip("; ")
            else:
                entry["c_abi"] = None
                entry["c_state"] = "mapped"
                entry["notes"] = (
                    (entry.get("notes") or "")
                    + ("; " if entry.get("notes") else "")
                    + "C export deferred to 0.9 (outside 0.8 preview allowlist)"
                ).strip("; ")

    OUT.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
