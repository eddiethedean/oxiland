# Oxiland 0.x roadmap

Status: active  
Applies to: Oxiland 0.1 through 0.10  
Companion plans: [architecture](ARCHITECTURE.md),
[compatibility](COMPATIBILITY.md), [verification](VERIFICATION.md), and
[execution](EXECUTION.md). Durable backend expansion is specified separately in
the [storage backend plan](design/storage-backend-expansion.md).

The 0.x series is compatibility-driven. Minor versions may change Rust APIs,
but every breaking change requires migration notes, and RDF/SPARQL behavior
must never change silently. Dates are intentionally omitted: a milestone ships
only when its evidence gates are satisfied.

## Release train

| Version | Outcome | Depends on | State |
|---|---|---|---|
| 0.1 | Trusted core model | — | `complete` |
| 0.2 | Redland-shaped RDF I/O | 0.1 | `complete` |
| 0.3 | Complete query/result workflows | 0.2 | `complete` |
| 0.4 | Durable storage and transactions | 0.3 | `complete` |
| 0.5 | Streams, utilities, and observability | 0.4 | `complete` |
| 0.6 | Accounted safe Rust parity | 0.5 | `complete` |
| 0.7 | Pythonic package on PyPI | 0.4 (sequenced after 0.6) | `complete` |
| 0.8 | Auditable C ABI preview | 0.6 | `complete` |
| 0.9 | Downstream C compatibility | 0.8 | `complete` |
| 0.10 | 100% Redland parity and faster-than-Redland release candidate | 0.9 | `planned` |

States are `planned`, `in progress`, `blocked`, or `complete`. A state changes
only after the evidence links are added to the root
[parity ledger](parity.md).

0.7 may begin design against the 0.4 storage contract, but it is sequenced after
0.6 so the Python surface maps a reviewed safe Rust API rather than a moving
facade. 0.8 (C ABI) remains gated on 0.6 independently of Python.

Storage backend expansion is a cross-cutting 0.8–0.10 track. 0.8 extracts and
proves a backend-neutral durable adapter before the C ABI freezes backend
selection; 0.9 implements and validates optional adapters; 0.10 stabilizes the
supported matrix and its migration guarantees. Fjall remains the default and
format-v1 compatibility baseline throughout this work.

## Rules for every milestone

Each release must:

- retain a compiling example of its primary workflow;
- update the API inventory and parity ledger;
- document additions, breaking changes, and known deviations;
- pass the release gates defined in the verification plan;
- record architecture or compatibility decisions that affect later phases;
- avoid placeholder APIs that imply support but return generic errors.

## 0.1 — Core model

Outcome: establish the safe Rust vocabulary and a dependable in-memory graph.

State: complete  
Evidence: [parity ledger](parity.md),
[0.1 compatibility report](reports/0.1.md),
[inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.1.json)

Deliverables:

- `World`, RDF terms, statements, models, contexts, and pattern matching.
- In-memory Oxigraph storage.
- Basic SPARQL query execution.
- Public error model and feature flags.
- Initial parity inventory and documentation.
- Experimental Fjall-backed persistence via `Model::open`, without a stable
  on-disk compatibility or transaction promise.
- Named-graph-aware removal and containment operations.
- Streaming `Model::find` via `StatementMatches` (ADR-005).
- Rustdoc examples showing construction, CRUD, contexts, and SPARQL.

Evidence gates:

- Core CRUD, duplicates, invalid input, and named-graph behavior have tests.
- Public APIs build without Clippy warnings.
- Rust 1.87 and stable Rust are tested.
- Every exposed item has Rustdoc documentation.
- 0.1 inventory rows cite implementation and test locations.
- Direct Oxigraph term re-exports (ADR-004) and streaming find (ADR-005) are
  accepted.

Not in this milestone: polished parser/serializer facades, stable durable-store
guarantees, transactions, backup/migration workflows, or a C ABI. The
experimental Fjall path is intentionally promoted to a supported storage
contract only in 0.4.

## 0.2 — RDF input and output

Outcome: match Redland parser and serializer workflows through safe,
stream-oriented APIs.

State: complete
Execution specification: [milestone 0.2](milestones/0.2.md)
Evidence: [parity ledger](parity.md),
[0.2 compatibility report](reports/0.2.md),
[inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.2.json)

Deliverables:

- Safe `Parser` and `Serializer` facades.
- Reader, writer, string, file, and base-IRI entry points.
- Syntax discovery by name, MIME type, and file extension.
- Namespace configuration and serializer features.
- Turtle, N-Triples, N-Quads, TriG, and RDF/XML where supported by Oxigraph.
- Explicit parser source, graph-target, and blank-node-scope semantics.
- Bounded streaming paths that do not require loading an entire document.

Evidence gates:

- Parser output/atomicity and format-discovery decisions are accepted.
- Round-trip tests exist for every supported syntax.
- Parser errors preserve useful source locations.
- Supported and unsupported Redland syntax names are documented.
- Base IRI, relative IRI, language tag, datatype, and malformed-input cases pass.
- Applicable W3C syntax conformance manifests run in Oxiland CI.
- At least one large-input test demonstrates bounded parser memory behavior.

Depends on: stable model insertion/context semantics from 0.1.

## 0.3 — Query and results

Outcome: provide complete Redland-style SPARQL workflows without forcing result
materialization.

State: complete
Execution specification: [milestone 0.3](milestones/0.3.md)
Evidence: [parity ledger](parity.md),
[0.3 compatibility report](reports/0.3.md),
[inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.3.json)

Deliverables:

- Parsed queries, base IRIs, limit, offset, and query features.
- Boolean, bindings, graph, and syntax result forms.
- Streaming solution and graph-result iterators.
- SPARQL Update and dataset selection.
- Result serialization in Redland-supported formats where practical.
- Variable lookup by name and position with explicit unbound semantics.
- Query cancellation/timeout policy, even if the initial policy is unsupported.

Evidence gates:

- Each query result kind has positive, empty, and failure-path tests.
- Query and update facade behavior is covered by inventory-linked tests and the
  SPARQL smoke harness (`classification: oxiland-facade`). Native Rasqal
  differential oracles remain deferred (see [0.3 report](reports/0.3.md)).
- Iterator lifetimes do not require materializing full result sets.
- Ordering is asserted only where SPARQL guarantees it.
- Dataset/default-graph behavior has facade tests; native differential fixtures
  expand when Rasqal oracles land.
- Query error categories preserve parse versus evaluation failures.

Depends on: 0.2 dataset loading and result serialization.

## 0.4 — Storage and transactions

Outcome: cover durable models and Redland storage semantics with explicit
backend capabilities.

State: complete
Execution specification: [milestone 0.4](milestones/0.4.md)
Evidence: [parity ledger](parity.md),
[0.4 compatibility report](reports/0.4.md),
[inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.4.json)

Deliverables:

- Stable memory and fjall storage constructors.
- Storage options with typed configuration.
- Transactions, sync, bulk loading, and graph clearing.
- Storage capability reporting.
- Import/export paths for legacy Redland stores where feasible.
- Locking, read-only, backup, and unsupported-option behavior.
- A per-legacy-backend decision record.

Evidence gates:

- Crash-safe persistence and reopen tests pass.
- Transaction commit and rollback tests pass.
- Unsupported legacy backends return explicit capability errors.
- Storage compatibility decisions are recorded in the parity ledger.
- Concurrent reader/writer behavior is tested and documented.
- A persistent store created by each supported prior Oxiland minor version can
  be opened or migrated.
- fjall remains the durable backend and the default build stays free of native C++ deps.

Depends on: stable model, I/O, and update behavior from 0.1–0.3.

## 0.5 — Streams, utilities, and observability

Outcome: complete the non-query high-level Rust surface needed for Redland
workflow parity.

State: complete  
Execution specification: [milestone 0.5](milestones/0.5.md)  
Design: [0.5-streams-utilities.md](design/0.5-streams-utilities.md)  
Release evidence:
[0.5 compatibility report](reports/0.5.md),
[inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.5.json)

Deliverables:

- Lazy statement streams and generic iterators.
- URI helpers, digests, filename/URI heuristics, and Unicode helpers.
- Logging facade with optional `tracing` integration.
- Namespace concepts and well-known RDF vocabulary constants.
- Standard-library replacements for Redland hashes and lists.
- Deterministic callback and iterator error behavior.
- Migration examples for APIs intentionally replaced by Rust primitives.

Evidence gates:

- Stream APIs are lazy and covered by early-termination tests.
- Utility output is differentially tested against Redland.
- No compatibility utility panics on malformed external input.
- Logging levels/facilities and callback ordering are fixture-tested.
- Every hash/list/manual-memory symbol has a documented Rust mapping or
  non-applicable rationale.

Depends on: the stable error and lifetime models exercised by 0.1–0.4.

## 0.6 — Safe Rust API parity

Outcome: account for the full public Redland API in a reviewed safe Rust
surface.

State: complete  
Evidence: [parity ledger](parity.md),
[0.6 compatibility report](reports/0.6.md),
[inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.6.json),
[milestone plan](milestones/0.6.md)

Deliverables:

- Finish the generated Redland **function** inventory (header-derived).
- Map every symbol to a Rust API, compatibility shim, or documented
  non-applicable ownership operation.
- Add `rdfproc`-equivalent command workflows.
- Stabilize feature and error semantics.
- Publish a Redland-to-Oxiland migration guide.
- Freeze naming and module conventions intended for 1.0.

Evidence gates:

- The inventory has no unclassified public symbols.
- All applicable safe-Rust mappings have tests.
- The parity ledger reports 100% safe-API accounting, not C ABI parity.
- An external API review finds no unresolved soundness or ownership issues.
- `cargo semver-checks` or an equivalent public-API snapshot is established.
- Every accepted deviation names impact, workaround, owner, and review date.

Depends on: all high-level safe API milestones.

## 0.7 — Python package

Outcome: ship a maintained PyPI package with **Pythonic** interfaces over the
safe Rust facade—not a mechanical 1:1 port of every Rust type and builder.

State: complete  
Evidence: [parity ledger](parity.md),
[0.7 compatibility report](reports/0.7.md),
[milestone plan](milestones/0.7.md),
[design](design/0.7-python-api.md), ADR-017

Deliverables:

- A separate installable package (working name `oxiland`, published to PyPI)
  built against the Rust crate (for example PyO3 / maturin), not against
  `oxiland-capi`.
- Idiomatic Python surface for models, terms, I/O, SPARQL query/update, and
  results: keyword arguments, properties, context managers where acquisition
  and release apply, and iterators that follow the iterator protocol.
- A documented exception hierarchy aligned with Oxiland error categories
  (not raw stringly Rust error text as the only API).
- Type hints and packaging that support static checkers (PEP 561 / stub or
  inline annotations).
- `pathlib.Path` / path-like acceptance for file entry points; buffer/bytes
  and text paths that match Python I/O norms.
- Streaming result and parse consumers that do not force full materialization
  when the Rust facade streams.
- Python examples and a standalone documentation track that teach installation,
  models, I/O, SPARQL, operations, and the API on Python's own terms.
- An explicit design note for what is *not* mirrored 1:1 from Rust (builders
  flattened to functions/kwargs, ownership differences, naming).
- Optional interop story evaluated and recorded (for example converting to/from
  common Python RDF types); first release may ship without rdflib integration
  if the ADR rejects it for scope.

Evidence gates:

- Wheel (or equivalent) builds and installs cleanly in CI on the published
  platform matrix.
- Pytest covers model CRUD, parse/serialize, ASK/SELECT/Update, and failure
  paths through the Python API.
- Public Python APIs have type information checked in CI.
- Docs and examples run as part of the Python package verification.
- The package does **not** claim CPython ABI stability tied to `oxiland-capi`,
  and does not present itself as a drop-in for legacy Redland Python bindings
  unless a later decision adds that claim with fixtures.
- A design/ADR records the Pythonic-vs-thin-binding boundary before the
  first public beta.

Not in this milestone: wrapping every Oxigraph or Redland Python API; a pure
ctypes/`cffi` binding of the C ABI; guaranteeing behavioral identity with
rdflib; or freezing the Python API for 1.0 before 0.10 soak.

Depends on: durable models and query/update from 0.3–0.4, and reviewed safe
Rust mappings from 0.6 before a non-experimental PyPI release.

## 0.8 — C ABI preview

Outcome: run representative existing C consumers against an auditable Oxiland
compatibility library.

State: complete  
Evidence: [parity ledger](parity.md),
[0.8 compatibility report](reports/0.8.md),
[inventory](https://github.com/eddiethedean/oxiland/blob/main/compatibility/inventory/redland-1.0.17-oxiland-0.8.json),
[milestone plan](milestones/0.8.md),
[design](design/0.8-cabi.md), ADR-022, ADR-023

Deliverables:

- A separate `oxiland-capi` workspace crate.
- A sealed durable-store adapter boundary with Fjall running through it; no
  backend-native type crosses the safe facade or C ABI.
- Typed backend selection and availability/capability discovery that can name a
  known but disabled optional adapter without treating it as an unknown store.
- Generated `librdf.h`-compatible declarations.
- Opaque handles, allocation rules, callbacks, and error translation.
- `cdylib` and `staticlib` artifacts.
- Symbol versioning and platform naming strategy.
- Panic containment and a documented thread-safety matrix.
- Installable headers and `pkg-config` metadata.

Evidence gates:

- A representative unmodified Redland C program compiles and runs.
- Sanitizers find no leaks, use-after-free, or callback lifetime defects.
- Exported-symbol checks run in CI on supported platforms.
- ABI limitations are documented prominently.
- Every exported pointer type has allocation, aliasing, and destruction tests.
- Null, invalid UTF-8, callback re-entry, and double-free defenses are tested.
- No `unsafe` block lacks a local safety argument.
- The existing Fjall format-v1 and storage transaction suites pass through the
  common backend conformance harness.
- The default build remains free of native C/C++ storage dependencies.

Depends on: 0.6 safe API accounting and stable ownership semantics.

## 0.9 — C compatibility and ecosystem validation

Outcome: prove broad source and behavioral compatibility using real consumers.

State: complete

Deliverables:

- Complete applicable C symbol implementations.
- First-party optional storage adapters for redb, RocksDB, SQLite, and LMDB,
  each behind an explicit Cargo feature and selected through the common API.
- Evidence-backed promote/defer/reject decisions for sled, LevelDB, MDBX, and
  SurrealKV; no evaluation backend is advertised as supported before it passes
  the common crash, reopen, transaction, and platform gates.
- Consistent backend selection and capability discovery in Rust, the CLI,
  Python, and C, plus standards-RDF and evaluated direct-copy migration paths.
- Differential harnesses that execute the same cases against both libraries.
- Builds of selected Redland language bindings and downstream applications.
- Performance and memory baselines.
- A published supported-platform and downstream-consumer matrix.
- Packaging smoke tests using installed rather than workspace artifacts.

Evidence gates:

- The C symbol inventory has no unexplained gaps.
- Selected downstream consumers pass their test suites unchanged.
- Known behavioral deviations are either fixed or accepted in published
  compatibility notes.
- No severity-high correctness or memory-safety defect remains open.
- Source and ABI claims are separately measured on each supported platform.
- Performance regressions above agreed budgets have decisions, not silent
  waivers.
- Every promoted storage adapter passes the shared conformance suite and its
  documented feature/platform build matrix.
- Opening a store with the wrong backend fails before any files are initialized
  or mutated.

Depends on: a sanitizer-clean 0.8 ABI preview.

## 0.10 — Release candidate

Outcome: reach 100% parity with the pinned Redland baseline, beat Redland on
the frozen performance matrix, then freeze and validate the design intended
for 1.0.

Deliverables:

- API and ABI stabilization (Rust, Python package, and C where promised).
- Freeze the supported storage backend identities, feature names,
  `StorageCapabilities`, and per-backend layout-reader policy intended for 1.0.
- Accept or reject a safe public custom-backend trait after the sealed adapter
  has been exercised by the first-party matrix.
- Cross-platform packaging and installation documentation.
- Upgrade guide for Redland users.
- Security, fuzzing, interoperability, and performance hardening.
- A reproducible, apples-to-apples Oxiland-versus-Redland benchmark suite for
  every supported target/build profile.
- Support, deprecation, MSRV, and vulnerability-response policies.
- Reproducible source archives and checksummed release artifacts.

Evidence gates:

- **Hard release gate:** the 0.10 release does not ship until the project meets
  the [100% Redland parity definition](COMPATIBILITY.md#010-full-redland-parity-gate)
  for Redland `librdf` 1.0.17 (manual 1.0.18) on every supported target and
  build profile.
- The generated parity report is exactly 100%: every in-scope public symbol is
  implemented and every applicable observable behavior is `verified`, with
  the numerator, denominator, skips, platform/profile, and evidence revision
  published. Mechanical ownership operations may be `not-applicable` only for
  the safe Rust mapping; their C ABI and observable lifecycle semantics must
  still be verified.
- There are zero in-scope `unreviewed`, `mapped`, `implemented`, or `excluded`
  inventory rows, zero unexplained differential mismatches, and zero accepted
  behavioral deviations. A waiver, quarantine, migration workaround, or
  capability error does not satisfy this gate.
- **Hard performance gate:** Oxiland must pass the
  [0.10 faster-than-Redland gate](VERIFICATION.md#010-faster-than-redland-gate)
  on every required benchmark and supported performance profile. No required
  case may tie or lose, and wins in other cases may not average away a loss.
- Rust public-API snapshots and C ABI snapshots are enforced in CI.
- Python package versioning and wheel matrix are documented and green.
- The full conformance and differential matrix is green.
- Documentation includes complete examples for supported Redland workflows.
- Release candidates receive real downstream testing.
- No release-blocking item remains in the risk register.
- A clean environment can install, link, execute, and uninstall every artifact.
- At least one release-candidate soak period completes without an ABI reset.
- Every supported durable layout has a tested reader/export path, and removing
  an adapter cannot strand the only readable copy of user data.

## 1.0 readiness

Version 1.0 is eligible only after 0.10 has passed both the 100% Redland parity
gate and the faster-than-Redland performance gate. The safe Rust mappings and
the promised C source, ABI, and behavioral surfaces must meet their published
definitions with no in-scope exclusion or deviation.
The Python package (if still in the 1.0 promise) must also meet its published
PyPI contract. Independent Raptor/Rasqal APIs not exposed through `librdf`,
third-party plug-ins outside the pinned baseline, and targets outside the
published support matrix are not part of the denominator; anything inside that
denominator is mandatory.

The release decision consumes the evidence defined above; elapsed time,
inventory percentages alone, or a green unit-test suite are insufficient.
