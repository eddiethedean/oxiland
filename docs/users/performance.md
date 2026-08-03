# Performance

## Highlight: every strict case won locally

The optimized tip beat genuine system Redland 1.0.17 in **all ten required
cases** on macOS/arm64. The corrected driver ran 100 per-sample AB/BA pairs per
case, calibrated each sample to at least 10 ms, and evaluated paired-bootstrap
95% confidence intervals. Higher is better for throughput; lower is better for
latency.

| Case | Oxiland / Redland | Paired 95% CI |
|---|---:|---:|
| Insert 1K | 3.312× | 3.289–3.356 |
| Insert 10K | 27.193× | 26.667–27.385 |
| Scan 10K | 2.077× | 2.064–2.091 |
| Parse Turtle 1K | 5.086× | 5.050–5.115 |
| Parse Turtle 10K | 29.350× | 29.090–29.670 |
| Serialize N-Quads 10K | 7.158× | 7.094–7.198 |
| ASK latency | 0.106× | 0.104–0.109 |
| SELECT 10K | 1.738× | 1.700–1.773 |
| CONSTRUCT 10K | 2.102× | 2.078–2.153 |
| 100K model-size calls | 1.598× | 1.595–1.602 |

Median peak RSS was 1.012× Redland for parsing and 1.246× for SELECT, both
inside the frozen 1.25 budget. Reproduce a host run with:

```console
python3 scripts/run-0.13-performance.py --output oxiland-0.13-performance.json
```

This is a strong **host-scoped** result, not yet a blanket cross-platform
claim. Linux, macOS, and Windows must each pass three independent runs before
the suite-wide claim is frozen.

## Release qualification baseline

Milestone [0.12](../milestones/0.12.md) originally froze a
**competitive-parity** gate
([ADR-028](../DECISIONS.md#adr-028-012-competitive-parity-performance-gate)):
on matched production builds, Oxiland must stay within about 10% of Redland on
every required case (throughput median ≥ `0.90`, latency ≤ `1.20`, with
bootstrap CI bounds).

!!! warning "Historical 0.12 evidence"

    The committed 0.12 evidence bundle predates the runtime-library isolation
    fix and must not be used for a faster-than-Redland claim. The strict result
    above comes from the corrected runner, which removes Oxiland's compatibility
    library path from every Redland process.

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
  and [ADR-028](../DECISIONS.md#adr-028-012-competitive-parity-performance-gate).
- Results and methodology: [0.12 report](../reports/0.12.md); strict candidate
  suite in `compatibility/performance/0.13-suite.json`.
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
