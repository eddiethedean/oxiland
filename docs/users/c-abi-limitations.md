# C ABI preview limitations

Oxiland 0.8 `oxiland-capi` is an **auditable source-compat preview**. Treat
these limits as product contract, not temporary footnotes.

## Not a Redland drop-in

- **Not ABI-compatible** with existing Redland (`librdf`) shared libraries.
  Recompile against Oxiland headers; do not expect binary interchange.
- **Not a complete `librdf` surface.** Only the frozen 0.8 allowlist is
  declared and exported. Other Redland symbols are absent from the preview
  header and deferred to 0.9.
- **Not a guarantee** that every Redland C program will compile unchanged.
  Programs that call symbols outside the allowlist will not link.

## Behavioral and API gaps

- CONSTRUCT/DESCRIBE graph results are not exposed on the preview
  `librdf_query_results_*` API (ASK and SELECT bindings are).
- World log-handler / full Redland factory registration surfaces are out of
  scope for the preview.
- Optional durable backends (redb, RocksDB, SQLite, LMDB, …) are recognized as
  known-but-not-compiled; only `memory` and `fjall` open successfully.
- Double-free after the allocator reuses an address remains undefined; the
  preview may detect some invalid second frees but does not make misuse safe.

## Ownership rules that differ from ad-hoc C practice

- Mix of system `free` with Oxiland allocations is unsupported.
- Stream and query-result handles are bound to their model lifetime; using
  them after model free is invalid.
- Thread-safety is per handle type—see the matrix in
  [design/0.8-cabi.md](../design/0.8-cabi.md). Do not assume Redland's
  historical concurrency habits.

## What 0.9 is for

Downstream consumer matrices, fuller symbol closure, packaging as an installed
ABI artifact, and optional storage adapters are **0.9** work. Until then,
integrate only against the documented allowlist and the
[C ABI guide](c-abi.md).
