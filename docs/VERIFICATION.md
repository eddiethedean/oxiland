# Verification and quality gates

Status: active quality contract  
Applies to: local development, CI, differential infrastructure, and releases

Verification answers three separate questions:

1. Does Oxiland implement its documented Rust API correctly?
2. Does it match the applicable RDF/SPARQL standards?
3. Does it reproduce the Redland behavior claimed by the compatibility plan?

A passing test in one layer does not substitute for the others.

## Test layers

| Layer | Purpose | Location |
|---|---|---|
| Unit | conversions, configuration, invariants, errors | module-local tests |
| Rust integration | public workflows and feature combinations | `tests/` |
| Conformance | RDF and SPARQL standards | `compatibility/conformance/` |
| Differential | Oxiland versus native Redland | `compatibility/fixtures/` |
| C contract *(planned 0.8+)* | headers, symbols, allocation, callbacks | `crates/oxiland-capi/tests/` |
| Python package | wheels, typing, pytest | `python/` |
| Downstream | real language bindings and applications | CI-managed manifests |
| Fuzz/property *(planned expansion)* | malformed inputs and lifecycle sequences | `fuzz/` |

Planned locations are created with the milestone that first needs them.

## Fixture requirements

Every compatibility fixture has:

- a stable ID matching one or more inventory entries;
- setup and cleanup that are isolated and deterministic;
- backend operations expressed without implementation-specific shortcuts;
- expected result category and normalization profile;
- assertions for both success and relevant failure paths;
- source/oracle version metadata.

Tests involving unordered results must not rely on incidental iteration order.
Network-dependent behavior is captured in hermetic fixtures or runs in a
separate explicitly non-hermetic suite.

## Differential harness

Fixtures should be data-driven so one case can execute through both backends.
Each case contains setup, operations, expected result class, and normalization
rules. Backend runners emit a common JSON result containing values, errors,
logs, and relevant state.

Differences are classified as:

- implementation defect;
- expected non-semantic formatting variation;
- undocumented Redland behavior that becomes part of the compatibility target;
- accepted deviation with a published rationale.

Golden outputs alone are insufficient because they can encode a mistaken
interpretation. Where possible, the native Redland runner is the oracle.

The harness produces machine-readable output and a human-readable diff. A
fixture passes only if both runners complete, normalization succeeds, and the
result comparison is equal. Crashes, timeouts, skipped runners, and missing
oracle metadata are not passes.

## Standards conformance

Relevant W3C manifests should be pinned and run through Oxiland's public API,
not only inherited from Oxigraph's upstream test claims. Expected upstream
deviations are recorded with version and issue links.

Conformance categories include:

- RDF term construction and equality;
- Turtle, TriG, N-Triples, N-Quads, and RDF/XML;
- SPARQL query, update, protocol-independent dataset semantics, and results;
- RDF 1.2/SPARQL 1.2 only when the corresponding Oxiland feature is promised.

## Safety and robustness

- Miri covers safe abstractions where practical.
- AddressSanitizer and LeakSanitizer cover the C ABI and native harness.
- UndefinedBehaviorSanitizer covers C shims where supported.
- Fuzzers retain a checked-in regression corpus.
- Panics are acceptable only for documented programmer invariants; malformed
  RDF, queries, files, options, or C inputs must not panic.
- Thread and callback tests include re-entry and concurrent destruction where
  the API permits them.

## CI matrix

Required on every PR and on `main`/release:

- stable Rust checks (fmt, Clippy, tests, docs, examples, inventory,
  documentation links, public-API snapshot);
- safe Rust workspace tests on Linux, macOS, and Windows;
- dedicated Fjall persistence tests;
- Rust 1.87 MSRV Clippy and tests;
- RustSec audits for the workspace and Python extension lockfiles;
- non-breaking semver compatibility against 0.6.0 (forced to minor-release
  policy for the pre-1.0 API) and packaged-crate verification;
- Python pytest, Pyright, and runnable examples;
- Linux, macOS, and Windows wheel builds for CPython 3.10–3.14;
- metadata, license, typing, native-extension, and CycloneDX SBOM validation
  for every wheel;
- clean install/import smoke tests for every produced wheel;
- RDF I/O conformance and compatibility harness smokes.

Planned broader coverage includes:

- sanitizer-enabled Linux/macOS C ABI tests;
- expanded license policy and generated dependency inventory checks.

Workflow permissions default to read-only, third-party Actions use immutable
full commit SHAs, and dependency updates arrive as grouped Dependabot pull
requests. Release jobs elevate only the individual permissions needed for OIDC
attestations or GitHub release assets. The Rust workspace and independent
Python extension each commit their `Cargo.lock`, making `--locked` builds and
RustSec results reproducible on clean runners.

Security advisories are blocking on tip CI, not release-only. Tip and release
both run `cargo audit` on `Cargo.lock` and `python/Cargo.lock` with **no**
advisory ignores. Tip also validates package-version alignment, `cargo publish
--dry-run` for the library crate, and the full 15-wheel release matrix after
per-OS install smokes. As of the 0.7.0 preflight, PyO3 is at 0.29.0, but
Oxigraph 0.5.9 still constrains `quick-xml` to 0.37 (RUSTSEC-2026-0194 /
RUSTSEC-2026-0195), so tip CI stays red until that upstream line can accept
`quick-xml >= 0.41`. See R-020 in the [risk register](RISKS.md).

Nightly or scheduled coverage includes:

- fuzzing and Miri;
- full W3C and differential suites;
- big-endian or cross-architecture checks when infrastructure permits;
- persistent-store crash and concurrency scenarios;
- downstream rebuilds and performance baselines.

Persistent storage tests use isolated temporary directories and verify reopen,
rollback, interrupted writes, and concurrent access behavior.

## Local gates

Before review:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo doc --workspace --all-features --no-deps --locked
python3 scripts/check-inventory.py
python3 scripts/check-docs.py
scripts/generate-public-api.sh check
```

For Python changes:

```text
cd python
python -m pip install --requirement requirements-ci.txt
maturin develop --locked
pytest -q
pyright
python examples/quick_start.py
python examples/select.py
python examples/parse_serialize.py
python examples/persistent.py
maturin build --release --locked
```

For documentation changes:

```text
python3 scripts/check-docs.py
python3 -m pip install --requirement docs/requirements.txt
python3 -m mkdocs build --strict
```

Milestones may add native or long-running commands to this baseline. Release
artifact checks must operate on the built crate, CLI, or wheel rather than only
the workspace source tree. Python release artifacts are the exact CI-built and
install-smoked wheels; release jobs do not rebuild them. Their metadata,
bundled licenses, PEP 561 files, native extension, and CycloneDX SBOM are
checked before provenance attestation and publication.

## Release gates by phase

Every 0.x release requires:

- all local gates on required platforms;
- updated parity and roadmap status;
- updated dependency audit and minimum Rust check;
- successful packaging and clean-install smoke tests;
- no unexplained regression in the milestone's differential suite.

Additional phase gates:

| Starting version | Added release blocker |
|---|---|
| 0.2 | applicable RDF syntax conformance |
| 0.3 | SPARQL query/update facade conformance and smoke harness (Rasqal differential expands later) |
| 0.4 | persistence, transaction, and reopen matrix |
| 0.6 | complete safe-API inventory and public-API snapshot |
| 0.7 | Python wheels, type checks, and pytest matrix |
| 0.8 | exported symbols, C examples, and sanitizers |
| 0.9 | selected downstream C consumers |
| 0.10 | Rust API, Python package, and C ABI snapshots plus RC soak |

Flaky tests are quarantined only with an owner, issue, expiry milestone, and a
replacement signal. Quarantined compatibility tests do not count as passing.

## Performance verification

Compatibility is primary, but accidental performance cliffs can make a
compatible API unusable. Benchmarks track:

- triple insert/remove and pattern scans;
- parsing and serialization throughput;
- query latency and result streaming;
- persistent reopen and bulk load;
- peak memory on large streams;
- C-call and callback overhead.

Budgets are established before 0.9 from representative workloads. Benchmark
noise does not block a release unless it exceeds a documented threshold over
repeated runs.

## Metrics

Track separately:

- inventory items reviewed, implemented, and verified;
- tests by Redland subsystem;
- standard conformance pass rates;
- known behavioral deviations;
- downstream projects passing;
- parser and FFI fuzzing time without findings.

A single percentage must not combine these categories. Doing so would obscure
whether apparent progress represents documentation, implementation, or actual
behavioral verification.

Each release publishes numerator, denominator, skipped count, and the exact
inventory/suite revision for every percentage.
