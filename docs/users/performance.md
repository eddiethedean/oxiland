# Performance

## Claims policy

Publish only results that name the suite, host (or CI matrix), build profile,
and statistical method. Do **not** treat a single-host table as a substitute
for the suite-wide claim below.

| Claim class | What it means | Where it lives |
|---|---|---|
| Competitive parity (ADR-028) | Oxiland stays within about 10% of Redland on every required case | 0.12 release gate / three-host bundle |
| Suite-wide faster-than-Redland (ADR-029) | Linux, macOS, and Windows each pass three independent strict runs | Tip evidence under `compatibility/qualification/performance/0.13/` + [0.13 report](../reports/0.13.md) |
| Host-scoped highlight | Corrected local run after library-path isolation | Highlight table below (macOS/arm64 only) |

Always measure **release** builds. Debug/`dev` compiles are not comparable to
Redland production libraries.

## Suite-wide faster-than-Redland (authorized)

Milestone [0.13](../milestones/0.13.md) closed [ADR-029](../DECISIONS.md#adr-029-013-suite-wide-faster-than-redland-gate):
Linux x86-64, macOS Apple Silicon, and Windows x86-64 each passed three
independent corrected-runner cells against
`compatibility/performance/0.13-suite.json` (throughput median ≥ `1.05` with
CI lower `> 1.0`; latency median ≤ `0.95` with CI upper `< 1.0`; 100 paired
samples; RSS ≤ `1.25×`). Qualifying CI:
[run 30973969324](https://github.com/eddiethedean/oxiland/actions/runs/30973969324)
on `a50ee5b25eb9daa56b0cf1d155856e1c312b35fb`.
`python3 scripts/check-0.13-release.py` is green on the committed bundle.

Worst-of-three run medians per host (minimum throughput ratio; maximum ASK
latency ratio):

| Case | Linux | macOS | Windows |
|---|---:|---:|---:|
| Insert 1K | 3.273× | 2.915× | 2.809× |
| Insert 10K | 40.152× | 25.298× | 32.678× |
| Scan 10K | 1.629× | 2.248× | 2.018× |
| Parse Turtle 1K | 3.692× | 5.148× | 3.074× |
| Parse Turtle 10K | 35.636× | 28.928× | 31.645× |
| Serialize N-Quads 10K | 7.096× | 6.348× | 9.738× |
| ASK latency | 0.314× | 0.286× | 0.216× |
| SELECT 10K | 1.564× | 2.518× | 2.039× |
| CONSTRUCT 10K | 2.715× | 5.462× | 4.009× |
| 100K model-size calls | 1.332× | 1.228× | 1.104× |

Full methodology and reproduce steps: [0.13 report](../reports/0.13.md).

```console
python3 scripts/check-0.13-release.py
```

## Highlight: every strict case won locally

Separately, an optimized tip beat genuine system Redland 1.0.17 in **all ten
required cases** on macOS/arm64. The corrected driver ran 100 per-sample AB/BA
pairs per case, calibrated each sample to at least 10 ms, and evaluated
paired-bootstrap 95% confidence intervals. Higher is better for throughput;
lower is better for latency. This remains a **host-scoped** result under the
claims policy above (the suite-wide claim is the three-host table).

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
inside the frozen 1.25 budget.

## Release qualification baseline

Milestone [0.12](../milestones/0.12.md) freezes a **competitive-parity** gate
([ADR-028](../DECISIONS.md#adr-028-012-competitive-parity-performance-gate)):
on matched production builds, Oxiland must stay within about 10% of Redland on
every required case (throughput median ≥ `0.90`, latency ≤ `1.20`, with
bootstrap CI bounds). Tip **0.12.0** closed that gate on the committed
three-host bundle. Milestone 0.13 then restored and closed the stricter
suite-wide margin (ADR-029) on the tip evidence above.

### Tip CI diagnostic medians (historical 0.12)

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
    isolation fix and must not be used for a faster-than-Redland claim. Prefer
    the suite-wide 0.13 table above (and the host-scoped highlight) for speed
    claims. The 0.12 tip CI Linux/macOS diagnostics come from the corrected
    runner but are not the ADR-029 nine-cell bundle.

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
  [VERIFICATION.md — 0.12 / 0.13 performance](../VERIFICATION.md#012-performance-optimization),
  [ADR-028](../DECISIONS.md#adr-028-012-competitive-parity-performance-gate), and
  [ADR-029](../DECISIONS.md#adr-029-013-suite-wide-faster-than-redland-gate).
- Results: [0.13 report](../reports/0.13.md) (suite-wide);
  [0.12 report](../reports/0.12.md) (competitive parity);
  strict suite in `compatibility/performance/0.13-suite.json`; CI gate in
  `.github/workflows/qualify-0.13.yml`.
- **Always measure release builds.** Use
  `cargo build -p oxiland-capi --release --locked` (and matching `--release`
  Rust examples/benches). Debug/`dev` compiles are not comparable to Redland
  production libraries and are rejected by the performance gates.

## Related guides

- [Streams and iterators](streams.md)
- [Persistence](persistence.md)
- [Rust production](rust-production.md)
- [Python production](python-production.md)
- [FAQ](faq.md)
- [Known limitations](limitations.md)
