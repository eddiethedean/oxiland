# C ABI

Oxiland 0.10 development ships a **source-compatible Redland-shaped** C library
(`oxiland-capi`) with an expanded allowlist. Use this surface to try a world →
storage → model → parse/serialize → ASK/SELECT workflow against Oxiland. It is
**not** a binary drop-in for `librdf`.

!!! warning "Preview limitations"
    Read [C ABI limitations](c-abi-limitations.md) before integrating. Source and Oxiland ABI claims are measured in 0.9; Redland `.so` swap is not claimed.

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

The default build enables Fjall. To compile an optional durable backend into
the C library, pass its matching feature, for example
`--features storage-sqlite` or `--features storage-lmdb`.

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

## Frozen development snapshot

Only the symbols declared in the shipped header are exported. Unsupported
Redland APIs are omitted rather than stubbed.

| Area | Symbols |
|---|---|
| World | construction, logging, and logger callbacks |
| Storage | construction, open, discovery, and sync |
| Model | CRUD, streams, sync, string export, SPARQL Update, and queries |
| Terms | URI, node, and statement construction, accessors, mutation, and string forms |
| I/O | parser/serializer names plus string, counted-string, and file variants |
| Query | ASK, SELECT, CONSTRUCT/DESCRIBE graph streams, and result iteration |
| Utilities | MD5/SHA digests, UTF-8/Latin-1 conversion, basename, and allocator release |

## Handle ownership

Opaque handles follow the contract in
[design/0.8-cabi.md](../design/0.8-cabi.md) (handle matrix, allocator, panic
containment):

- Free handles only with the matching `librdf_free_*`.
- Free library-returned strings only with `librdf_free_memory`.
- `NULL` free is a no-op; every `extern "C"` entry contains panics via
  `catch_unwind`.

## Storage names

`librdf_new_storage` accepts `"memory"`, `"fjall"`, and any optional backend
compiled into `oxiland-capi` (`"redb"`, `"rocksdb"`, `"sqlite"`, or `"lmdb"`;
durable names use `name` as their path). Known optional backends that are not
compiled fail with an explicit unsupported message distinct from unknown
names—the same registry as Rust `StorageBackend::from_name`.

## Evidence

Compatibility claims for this surface are scoped in the
[0.9 report](../reports/0.9.md), the
[0.10 qualification report](../reports/0.10.md), and the
[parity ledger](https://github.com/eddiethedean/oxiland/blob/main/PARITY.md).
