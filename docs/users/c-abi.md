# C ABI

Oxiland 0.11 ships a **source-compatible Redland-shaped** C library
(`oxiland-capi`, version `0.11.0`, `publish = false`) with a frozen allowlist.
Use this surface for world → storage → model → parse/serialize → ASK/SELECT
workflows against Oxiland. Milestone 0.11 also packages a **Redland-compatible
shared library name** (`librdf.0` / `librdf.so.0`) for binary ABI interchange
experiments; full parity still requires the qualification gate.

!!! warning "Preview limitations"
    Read [C ABI limitations](c-abi-limitations.md) before integrating. Source corpus and ABI swap scripts live under `compatibility/downstream/`; treat interchange results as evidence, not a completed 0.11 claim until the checker passes.

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
| Compat shared library (macOS) | `target/release/compat/librdf.0.dylib` (+ `librdf.dylib`) |
| Compat shared library (ELF) | `target/release/compat/librdf.so.0` (+ `librdf.so`) |
| Header | `crates/oxiland-capi/include/librdf.h` |
| pkg-config template | `crates/oxiland-capi/oxiland.pc.in` |
| Drop-in Redland pkg-config | `crates/oxiland-capi/librdf-compat.pc.in` |
| Symbol version script (ELF) | `crates/oxiland-capi/symbols.version` (`OXILAND_0.11` + `LIBRDF_1.0.17`) |

Debug builds place the same libraries under `target/debug/`.

### Redland-compatible packaging (0.11)

```console
cargo build -p oxiland-capi --release
scripts/package-librdf-compat.sh
```

On macOS the script sets `@rpath/librdf.0.dylib` via `install_name_tool`. On
Linux it sets soname `librdf.so.0` with `patchelf` when available. Point
`DYLD_LIBRARY_PATH` / `LD_LIBRARY_PATH` (or pkg-config) at
`target/release/compat` for load-without-rebuild checks:

```console
compatibility/downstream/abi/run-abi-swap.sh
compatibility/downstream/corpus/run-corpus.sh
```

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
2. Substitute `@PREFIX@` and `@VERSION@` (use `0.11.0` for the C API crate).
3. Put that directory on `PKG_CONFIG_PATH` and query flags:

```console
sed 's|@PREFIX@|'"$PWD"'/stage|g; s|@VERSION@|0.11.0|g' \
  crates/oxiland-capi/oxiland.pc.in > ./oxiland.pc
export PKG_CONFIG_PATH="$PWD:${PKG_CONFIG_PATH:-}"
pkg-config --cflags --libs oxiland
```

The template ships `Name: oxiland` and links `-loxiland_capi`.

For a Redland drop-in name (`-lrdf`), use `librdf-compat.pc.in` (or the
`redland.pc` written by `scripts/package-librdf-compat.sh` under
`target/release/compat/`).

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
