# oxiland-capi

C ABI preview for Oxiland. Redland-shaped headers and symbols are enforced
against a frozen snapshot. Full Redland ABI/behavioral parity remains a hard,
currently unmet 0.10 release gate.

**Canonical install and link guide:**
[`docs/users/c-abi.md`](../../docs/users/c-abi.md).

Design and milestone notes: `docs/design/0.8-cabi.md`,
`docs/design/0.9-cabi.md`, and `docs/milestones/0.10.md`.

This crate has `publish = false` and is **not** on crates.io. Build from a
clone of this repository.

## Build (repository root)

```console
cargo build -p oxiland-capi --release
```

Produces `target/release/liboxiland_capi.{a,so,dylib}` (crate-type `cdylib` +
`staticlib`). Link with `-loxiland_capi`.

## Headers and pkg-config

- Header: `crates/oxiland-capi/include/librdf.h`
- Template: `crates/oxiland-capi/oxiland.pc.in` (`Name: oxiland`)

Copy the `.pc.in` template, substitute `@PREFIX@`, and set `PKG_CONFIG_PATH`.
See [`docs/users/c-abi.md`](../../docs/users/c-abi.md) for the full recipe.

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
[`docs/users/c-abi.md`](../../docs/users/c-abi.md). Optional GNU ld version
script: `symbols.version`.
