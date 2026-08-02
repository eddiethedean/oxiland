# oxiland-capi

C ABI for Oxiland (`0.11.0`, `publish = false`). Redland-shaped headers and
symbols are enforced against a frozen snapshot. Source corpus and binary ABI
interchange packaging are active 0.11 work packages
(`compatibility/downstream/corpus/`, `scripts/package-librdf-compat.sh`).

**Canonical install and link guide:**
[`docs/users/c-abi.md`](../../docs/users/c-abi.md).

Design and milestone notes: `docs/design/0.8-cabi.md`,
`docs/design/0.9-cabi.md`, and `docs/milestones/0.11.md`.

This crate has `publish = false` and is **not** on crates.io. Build from a
clone of this repository.

## Build (repository root)

```console
cargo build -p oxiland-capi --release
scripts/package-librdf-compat.sh
```

Produces `target/release/liboxiland_capi.{a,so,dylib}` (crate-type `cdylib` +
`staticlib`) and Redland-compatible names under `target/release/compat/`
(`librdf.0.dylib` / `librdf.so.0`). Link with `-loxiland_capi` or `-lrdf`
against the compat directory.

## Headers and pkg-config

- Header: `crates/oxiland-capi/include/librdf.h`
- Template: `crates/oxiland-capi/oxiland.pc.in` (`Name: oxiland`)
- Drop-in: `crates/oxiland-capi/librdf-compat.pc.in` (`Name: Redland`, `-lrdf`)

Copy the `.pc.in` template, substitute `@PREFIX@` / `@VERSION@`, and set
`PKG_CONFIG_PATH`. See [`docs/users/c-abi.md`](../../docs/users/c-abi.md).

## Compile example (repository root)

```console
cargo build -p oxiland-capi
cc -I crates/oxiland-capi/include -L target/debug \
  crates/oxiland-capi/examples/preview_workflow.c \
  -loxiland_capi -Wl,-rpath,$PWD/target/debug \
  -o preview_workflow
```

Release variant uses `-L target/release` and
`-Wl,-rpath,$PWD/target/release`.

## Ownership

- Opaque handles: typed `librdf_free_*`; `NULL` free is a no-op.
- Strings from the library: free only with `librdf_free_memory`.
- Double-free of a non-null handle is undefined after the allocator reuses the
  address; the preview tries to detect a second free of an unregistered pointer.
- Every `extern "C"` entry contains panics via `catch_unwind`.

## Symbol allowlist

The frozen symbol table is embedded in
[`docs/users/c-abi.md`](../../docs/users/c-abi.md). GNU ld version
script: `symbols.version` (`OXILAND_0.11` with `LIBRDF_1.0.17` alias).
