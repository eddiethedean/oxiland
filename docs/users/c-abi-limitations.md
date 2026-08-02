# C ABI limitations

Oxiland 0.11 ships a Redland-shaped C surface (`oxiland-capi`) and a
librdf-compatible packaging path. The following limits remain part of the
product contract even after the 0.11 checker passes.

## Compatibility boundary

- **Source compatibility** is proven for the frozen C corpus and selected
  downstream consumers under `-Werror` against both Redland and Oxiland headers.
- **Binary interchange** is proven via the librdf-compat shared library and
  ABI-swap smoke on the qualification matrix. Programs outside that corpus may
  still need rebuilds if they depend on undocumented layouts or out-of-baseline
  plugins.
- Symbols outside the frozen inventory remain omitted; programs that call them
  will not link.

## Behavioral and API gaps

- CONSTRUCT/DESCRIBE are available via `librdf_query_results_is_graph` /
  `librdf_query_results_as_stream`.
- `librdf_world_set_logger` invokes the registered callback for messages
  emitted through `librdf_log_simple` / `librdf_log`. Broader Redland log
  facilities (Raptor locator-rich messages, per-level handlers with `va_list`)
  remain simplified.
- `librdf_*_register_factory` stores the factory function pointer on the world,
  invokes it at registration time, and consults it when creating an unknown
  named parser/serializer/storage/query. Custom factories do **not** install a
  full Redland vtable; parse/serialize/query still run on Oxiland built-ins
  (Turtle/SPARQL/memory fallbacks).
- iostream arguments (`raptor_iostream*` in Redland headers) are **not** Raptor
  objects. Non-null iostream pointers must be Oxiland tagged handles created
  with `oxiland_new_iostream` / `oxiland_new_iostream_from_bytes` (Rust helpers
  in `oxiland_capi`). Unknown non-null pointers fail; null write sinks succeed.
- Optional durable backends open when compiled (`storage-redb`,
  `storage-rocksdb`, `storage-sqlite`, `storage-lmdb`); otherwise they remain
  known-but-not-compiled.
- Double-free after the allocator reuses an address remains undefined; the
  library may detect some invalid second frees but does not make misuse safe.

## Ownership rules that differ from ad-hoc C practice

- Mix of system `free` with Oxiland allocations is unsupported.
- Stream and query-result handles are bound to their model lifetime; using
  them after model free is invalid.
- Thread-safety is per handle type—see the matrix in
  [design/0.8-cabi.md](../design/0.8-cabi.md). Do not assume Redland's
  historical concurrency habits.

## 0.11 status

Milestone 0.11 is complete: unchanged-source corpus builds, librdf-compat
packaging, and six-cell native differentials pass
`scripts/check-0.11-release.py`. See the
[downstream matrix](https://github.com/eddiethedean/oxiland/blob/main/compatibility/downstream/README.md),
the [C ABI guide](c-abi.md), and [`docs/reports/0.11.md`](../reports/0.11.md).

Claims must not exceed the verified matrix (Linux x86-64, macOS Apple Silicon,
Windows x86-64 × `release-default` / `release-all-storage`).
