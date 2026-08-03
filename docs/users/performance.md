# Performance

Milestone [0.12](../milestones/0.12.md) freezes a **competitive-parity** gate
([ADR-028](../DECISIONS.md#adr-028--012-competitive-parity-performance-gate)):
on matched production builds, Oxiland must stay within about 5% of Redland on
every required case (throughput median ≥ `0.95`, latency ≤ `1.05`, with
bootstrap CI bounds). That is **not** a blanket “faster than Redland” claim—
cite per-case ratios from the native matrix when discussing speed.

## Practical defaults

- Prefer streaming parse and serialize APIs for large files instead of loading
  entire documents as strings when the API offers a stream or path path.
- Consume SPARQL and `find` iterators incrementally; avoid collecting unbounded
  results into lists.
- Persistent (Fjall) models keep a full in-memory Oxigraph working set for
  query—plan RAM for the dataset size in addition to on-disk footprint.
- Apply SPARQL `LIMIT` / application budgets before exposing query to untrusted
  or latency-sensitive callers.
- Parse and bulk-load iterators do not expose wall-clock cancellation; stop
  consuming them or isolate the work at the process/thread boundary.

## Comparison status

- Protocol and thresholds:
  [VERIFICATION.md — 0.12 performance optimization](../VERIFICATION.md#012-performance-optimization)
  and [ADR-028](../DECISIONS.md#adr-028--012-competitive-parity-performance-gate).
- Results: [0.12 report](../reports/0.12.md); raw samples under
  `compatibility/qualification/performance/0.12/`.
- **Always measure release builds.** Use
  `cargo build -p oxiland-capi --release --locked` (and matching `--release`
  Rust examples/benches). Debug/`dev` compiles are not comparable to Redland
  production libraries and are rejected by the 0.12 performance gate.

## Related guides

- [Streams and iterators](streams.md)
- [Persistence](persistence.md)
- [Rust production](rust-production.md)
- [Python production](python-production.md)
- [FAQ](faq.md)
