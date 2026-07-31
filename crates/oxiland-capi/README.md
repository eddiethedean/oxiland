# oxiland-capi

C ABI preview for Oxiland (milestone 0.8). Redland-shaped headers and symbols
for a frozen allowlist — full ABI compatibility remains 0.9.

See `docs/design/0.8-cabi.md` and `docs/milestones/0.8.md`.

## Build

```console
cargo build -p oxiland-capi
```

Produces `liboxiland_capi.{a,so,dylib}` (crate-type `cdylib` + `staticlib`).
Link with `-loxiland_capi` (the cdylib is `liboxiland_capi`).

## Headers and pkg-config

- Header: `include/librdf.h`
- Template: `oxiland.pc.in` (`Name: oxiland`)

```console
cc -I include -L target/debug examples/preview_workflow.c -loxiland_capi -o preview_workflow
```

On macOS you may also need `-Wl,-rpath,$(pwd)/target/debug`.

## Ownership

- Opaque handles: typed `librdf_free_*`; `NULL` free is a no-op.
- Strings from the library: free only with `librdf_free_memory`.
- Double-free of a non-null handle is undefined after the allocator reuses the
  address; the preview tries to detect a second free of an unregistered pointer.
- Every `extern "C"` entry contains panics via `catch_unwind`.

## Symbol allowlist

Optional GNU ld version script: `symbols.version`.
