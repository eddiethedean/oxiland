# Performance

Oxiland does not yet authorize a blanket faster-than-Redland claim. Milestone
[0.12](../milestones/0.12.md) is the active optimization phase that closes
the frozen comparison gate; until it completes, treat the following as
operational guidance and cite only per-case native ratios when discussing
speed.

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
  [VERIFICATION.md — 0.12 performance optimization](../VERIFICATION.md#012-performance-optimization).
- Progress: [0.12 report](../reports/0.12.md).
- **Always measure release builds.** Use
  `cargo build -p oxiland-capi --release --locked` (and matching `--release`
  Rust examples/benches). Debug/`dev` compiles are not comparable to Redland
  production libraries and are rejected by the 0.12 performance gate.
- Native 0.11 diagnostic samples live under
  `compatibility/qualification/performance/` (`synthetic: false`). They show
  strong Unix wins on mutation/parse/serialize and known cliffs on scan and
  trivial C-call overhead; they are not a completed 0.12 gate.

## Related guides

- [Streams and iterators](streams.md)
- [Persistence](persistence.md)
- [Rust production](rust-production.md)
- [Python production](python-production.md)
- [FAQ](faq.md)
