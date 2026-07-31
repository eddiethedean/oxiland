# C ABI preview

Oxiland 0.8 ships a **source-compat preview** C library (`oxiland-capi`) with a
frozen Redland-shaped allowlist. Use this surface to try a world → storage →
model → parse/serialize → ASK/SELECT workflow against Oxiland. It is **not** a
binary drop-in for `librdf`.

!!! warning "Preview limitations"
    Read [C ABI limitations](c-abi-limitations.md) before integrating. Full
    symbol closure and ABI claims are planned for 0.9.

## Build the library

From the repository root:

```console
cargo build -p oxiland-capi --release
```

Artifacts:

- `target/release/liboxiland_capi.a` (static)
- `target/release/liboxiland_capi.dylib` / `.so` (shared; platform-dependent)

## Headers and pkg-config

| Item | Path |
|---|---|
| Header | `crates/oxiland-capi/include/librdf.h` |
| pkg-config template | `crates/oxiland-capi/oxiland.pc.in` |
| Symbol version script (ELF) | `crates/oxiland-capi/symbols.version` |

Install the header where your toolchain expects it (or pass `-I`), and fill
`@PREFIX@` in the `.pc.in` template for `pkg-config --cflags --libs oxiland`.

Link with `-loxiland_capi`. On macOS you may need an rpath to the library
directory.

## Quick example

A representative workflow lives at
`crates/oxiland-capi/examples/preview_workflow.c`:

```console
cargo build -p oxiland-capi
cc -I crates/oxiland-capi/include -L target/debug \
  crates/oxiland-capi/examples/preview_workflow.c \
  -loxiland_capi -o preview_workflow
```

## Preview allowlist

Only the symbols listed in the [0.8 milestone](../milestones/0.8.md) frozen
allowlist are exported. Unsupported Redland APIs are omitted from the preview
header rather than stubbed.

Covered areas: world, memory/fjall storage, model CRUD + find, URI/node/
statement, streams, Turtle-oriented parse/serialize checks, SPARQL ASK/SELECT
bindings, and `librdf_free_memory`.

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
