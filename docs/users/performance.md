# Performance

## Claims policy

Publish only results that name the suite, host (or CI matrix), build profile,
and statistical method. Do **not** treat any single-host table as a blanket
cross-platform or “always faster than Redland” claim.

| Claim class | What it means | Where it lives |
|---|---|---|
| Competitive parity (ADR-028) | Oxiland stays within about 10% of Redland on every required case | 0.12 release gate / three-host bundle |
| Host-scoped strict wins | Corrected local or tip-CI runs after library-path isolation | Highlight table and tip CI diagnostics below |
| Suite-wide faster-than-Redland | Linux, macOS, and Windows each pass three independent runs | Still open; not authorized by tip 0.12.0 |

Always measure **release** builds. Debug/`dev` compiles are not comparable to
Redland production libraries.

## Highlight: every strict case won locally

The optimized tip beat genuine system Redland 1.0.17 in **all ten required
cases** on macOS/arm64. The corrected driver ran 100 per-sample AB/BA pairs per
case, calibrated each sample to at least 10 ms, and evaluated paired-bootstrap
95% confidence intervals. Higher is better for throughput; lower is better for
latency. This is a **host-scoped** result under the claims policy above.

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
inside the frozen 1.25 budget. Reproduce a host run with the next-suite (0.13)
driver against the strict candidate suite:

```console
python3 scripts/run-0.13-performance.py --output oxiland-0.13-performance.json
```

The 0.13 script and `compatibility/performance/0.13-suite.json` are the
forward tooling for suite-wide qualification; they do not redefine the closed
0.12 competitive-parity gate. Linux, macOS, and Windows must each pass three
independent runs before a suite-wide faster-than-Redland claim is frozen.

## Release qualification baseline

Milestone [0.12](../milestones/0.12.md) freezes a **competitive-parity** gate
([ADR-028](../DECISIONS.md#adr-028-012-competitive-parity-performance-gate)):
on matched production builds, Oxiland must stay within about 10% of Redland on
every required case (throughput median ≥ `0.90`, latency ≤ `1.20`, with
bootstrap CI bounds). Tip **0.12.0** closes that gate on the committed
three-host bundle.

### Tip CI diagnostic medians

From
[0.12 Qualification run 30848514245](https://github.com/eddiethedean/oxiland/actions/runs/30848514245)
(post library-path isolation). Full table and footnotes:
[0.12 report](../reports/0.12.md#tip-ci-diagnostic-ratios-post-isolation).

| Case | Linux | macOS |
|---|---:|---:|
| Insert 1K | 3.808× | 3.771× |
| Insert 10K | 46.177× | 29.731× |
| Parse Turtle 10K | 37.689× | 31.858× |
| Serialize N-Quads 10K | 7.109× | 6.629× |
| ASK latency | 0.280× | 0.346× |
| 100K model-size calls | 1.447× | 0.818× |

!!! warning "Historical 0.12 evidence"

    The committed 0.12 release-gate bundle predates the runtime-library
    isolation fix and must not be used for a faster-than-Redland claim. The
    strict local result above and the tip CI Linux/macOS diagnostics come from
    the corrected runner, which removes Oxiland's compatibility library path
    from every Redland process. Windows tip CI in that run re-uploaded the
    committed historical bundle rather than a fresh isolated measurement.

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
- [Known limitations](limitations.md)
