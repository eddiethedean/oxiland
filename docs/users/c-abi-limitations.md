# C ABI preview limitations

Oxiland 0.10 development `oxiland-capi` is an **auditable Redland-shaped C surface**. Treat
these limits as product contract, not temporary footnotes.

## Not a Redland drop-in

- **Not ABI-compatible** with existing Redland (`librdf`) shared libraries.
  Recompile against Oxiland headers; do not expect binary interchange until
  packaging and binary-ABI gates in milestone 0.11 complete.
- **Not a complete historical `librdf` surface.** Symbols outside the allowlist
  are omitted or excluded with inventory notes.
- **Not a guarantee** that every Redland C program will compile unchanged.
  Programs that call symbols outside the allowlist will not link.

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
  preview may detect some invalid second frees but does not make misuse safe.

## Ownership rules that differ from ad-hoc C practice

- Mix of system `free` with Oxiland allocations is unsupported.
- Stream and query-result handles are bound to their model lifetime; using
  them after model free is invalid.
- Thread-safety is per handle type—see the matrix in
  [design/0.8-cabi.md](../design/0.8-cabi.md). Do not assume Redland's
  historical concurrency habits.

## 0.11 status

Downstream consumer matrices, expanded symbol surface, installed-artifact
packaging smokes, and optional storage adapters shipped in 0.9. See
the [downstream matrix](https://github.com/eddiethedean/oxiland/blob/main/compatibility/downstream/README.md)
and the [C ABI guide](c-abi.md).

Full parity remains blocked because the complete Redland source, binary ABI,
and behavior denominator is not verified from native, revision-bound evidence.
Milestone 0.11 requires unchanged-source builds and Redland-built binaries to
pass against Oxiland on every supported target; the shipped header remains a
preview until those gates pass.
