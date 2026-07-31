# C ABI preview

Oxiland 0.8 ships a **source-compat preview** C library (`oxiland-capi`) with a
frozen Redland-shaped allowlist. Use this surface to try a world → storage →
model → parse/serialize → ASK/SELECT workflow against Oxiland. It is **not** a
binary drop-in for `librdf`.

!!! warning "Preview limitations"
    Read [C ABI limitations](c-abi-limitations.md) before integrating. Full
    symbol closure and ABI claims are planned for 0.9.

## Prerequisites

- Rust **1.87+** (edition and MSRV used by the workspace)
- A C compiler (`cc` / Clang / GCC)
- A clone of **this repository** — `oxiland-capi` has `publish = false` and is
  **not** on crates.io

## Build the library

From the repository root:

```console
cargo build -p oxiland-capi --release
```

Artifacts (paths relative to the repo root):

| Artifact | Path |
|---|---|
| Static library | `target/release/liboxiland_capi.a` |
| Shared library (macOS) | `target/release/liboxiland_capi.dylib` |
| Shared library (ELF) | `target/release/liboxiland_capi.so` |
| Header | `crates/oxiland-capi/include/librdf.h` |
| pkg-config template | `crates/oxiland-capi/oxiland.pc.in` |
| Symbol version script (ELF) | `crates/oxiland-capi/symbols.version` |

Debug builds place the same libraries under `target/debug/`.

## Compile and link (repo root)

Canonical one-liner from the **repository root** (debug build + example):

```console
cargo build -p oxiland-capi
cc -I crates/oxiland-capi/include -L target/debug \
  crates/oxiland-capi/examples/preview_workflow.c \
  -loxiland_capi -Wl,-rpath,$PWD/target/debug \
  -o preview_workflow
```

Release variant:

```console
cargo build -p oxiland-capi --release
cc -I crates/oxiland-capi/include -L target/release \
  crates/oxiland-capi/examples/preview_workflow.c \
  -loxiland_capi -Wl,-rpath,$PWD/target/release \
  -o preview_workflow
```

`-Wl,-rpath,$PWD/target/debug` (or `.../release`) is the usual macOS pattern so
the dynamic loader finds `liboxiland_capi.dylib` without installing it. On Linux
you may use `-Wl,-rpath,$PWD/target/debug` the same way, or set
`LD_LIBRARY_PATH`.

Link with `-loxiland_capi` (the cdylib / staticlib name is `liboxiland_capi`).

## pkg-config

1. Copy `crates/oxiland-capi/oxiland.pc.in` to a writable location (for
   example `./oxiland.pc`).
2. Substitute `@PREFIX@` with the install prefix that contains `include/` and
   `lib/` (or point `prefix` at a staging directory you layout yourself).
3. Put that directory on `PKG_CONFIG_PATH` and query flags:

```console
cp crates/oxiland-capi/oxiland.pc.in ./oxiland.pc
# edit ./oxiland.pc: replace @PREFIX@ with your prefix
export PKG_CONFIG_PATH="$PWD:${PKG_CONFIG_PATH:-}"
pkg-config --cflags --libs oxiland
```

The template ships `Name: oxiland` and links `-loxiland_capi`.

## Preview allowlist

Only these symbols are exported in the 0.8 preview. Unsupported Redland APIs
are omitted from the preview header rather than stubbed.

| Area | Symbols |
|---|---|
| World | `librdf_new_world`, `librdf_free_world`, `librdf_world_open` |
| Storage | `librdf_new_storage`, `librdf_free_storage`, `librdf_storage_open` |
| Model | `librdf_new_model`, `librdf_free_model`, `librdf_model_add_statement`, `librdf_model_remove_statement`, `librdf_model_contains_statement`, `librdf_model_size`, `librdf_model_find_statements` |
| Terms | `librdf_new_uri`, `librdf_free_uri`, `librdf_new_node_from_uri_string`, `librdf_new_node_from_literal`, `librdf_free_node`, `librdf_new_statement_from_nodes`, `librdf_free_statement` |
| Stream | `librdf_stream_end`, `librdf_stream_next`, `librdf_stream_get_object`, `librdf_free_stream` |
| Parser | `librdf_new_parser`, `librdf_free_parser`, `librdf_parser_check_name`, `librdf_parser_parse_string_into_model` |
| Serializer | `librdf_new_serializer`, `librdf_free_serializer`, `librdf_serializer_check_name`, `librdf_serializer_serialize_model_to_string` |
| Query | `librdf_new_query`, `librdf_free_query`, `librdf_model_query_execute`, `librdf_query_results_is_boolean`, `librdf_query_results_get_boolean`, `librdf_query_results_is_bindings`, `librdf_query_results_finished`, `librdf_query_results_next`, `librdf_query_results_get_binding_name`, `librdf_query_results_get_binding_value`, `librdf_query_results_get_bindings_count`, `librdf_free_query_results` |
| Alloc | `librdf_free_memory` |

## Handle ownership

Opaque handles follow the contract in
[design/0.8-cabi.md](../design/0.8-cabi.md) (handle matrix, allocator, panic
containment):

- Free handles only with the matching `librdf_free_*`.
- Free library-returned strings only with `librdf_free_memory`.
- `NULL` free is a no-op; every `extern "C"` entry contains panics via
  `catch_unwind`.

## Storage names

`librdf_new_storage` accepts `"memory"` and `"fjall"` (path in `name`). Known
optional backends that are not compiled fail with an explicit unsupported
message distinct from unknown names—the same registry as Rust
`StorageBackend::from_name`.

## Evidence

Compatibility claims for this surface are scoped in the
[0.8 report](../reports/0.8.md) and
[parity ledger](https://github.com/eddiethedean/oxiland/blob/main/PARITY.md).
