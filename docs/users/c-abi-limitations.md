# C ABI preview limitations

Oxiland 0.9 `oxiland-capi` is an **auditable Redland-shaped C surface**. Treat
these limits as product contract, not temporary footnotes.

## Not a Redland drop-in

- **Not ABI-compatible** with existing Redland (`librdf`) shared libraries.
  Recompile against Oxiland headers; do not expect binary interchange.
- **Not a complete historical `librdf` surface.** Symbols outside the 0.9
  allowlist are omitted or excluded with inventory notes.
- **Not a guarantee** that every Redland C program will compile unchanged.
  Programs that call symbols outside the allowlist will not link.

## Behavioral and API gaps

- CONSTRUCT/DESCRIBE are available via `librdf_query_results_is_graph` /
  `librdf_query_results_as_stream`.
- `librdf_world_set_logger` receives only messages emitted through
  `librdf_log_simple`; full Redland factory registration and logging surfaces
  remain out of scope.
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

## 0.9 status

Downstream consumer matrices, expanded symbol surface, installed-artifact
packaging smokes, and optional storage adapters shipped in 0.9. See
the [downstream matrix](https://github.com/eddiethedean/oxiland/blob/main/compatibility/downstream/README.md)
and the [C ABI guide](c-abi.md).
