# Oxiland 0.x roadmap

Status: active  
Applies to: Oxiland 0.1 through 0.9  
Companion plans: [architecture](ARCHITECTURE.md),
[compatibility](COMPATIBILITY.md), [verification](VERIFICATION.md), and
[execution](EXECUTION.md)

The 0.x series is compatibility-driven. Minor versions may change Rust APIs,
but every breaking change requires migration notes, and RDF/SPARQL behavior
must never change silently. Dates are intentionally omitted: a milestone ships
only when its evidence gates are satisfied.

## Release train

| Version | Outcome | Depends on | State |
|---|---|---|---|
| 0.1 | Trusted core model | — | In progress |
| 0.2 | Redland-shaped RDF I/O | 0.1 | Planned |
| 0.3 | Complete query/result workflows | 0.2 | Planned |
| 0.4 | Durable storage and transactions | 0.3 | Planned |
| 0.5 | Streams, utilities, and observability | 0.4 | Planned |
| 0.6 | Accounted safe Rust parity | 0.5 | Planned |
| 0.7 | Auditable C ABI preview | 0.6 | Planned |
| 0.8 | Downstream C compatibility | 0.7 | Planned |
| 0.9 | 1.0 release candidate | 0.8 | Planned |

States are `planned`, `in progress`, `blocked`, or `complete`. A state changes
only after the evidence links are added to the root
[parity ledger](../PARITY.md).

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

Deliverables:

- `World`, RDF terms, statements, models, contexts, and pattern matching.
- In-memory Oxigraph storage.
- Basic SPARQL query execution.
- Public error model and feature flags.
- Initial parity inventory and documentation.
- Named-graph-aware removal and containment operations.
- Borrowed or lazy pattern iteration, or a documented temporary eager boundary.
- Rustdoc examples showing construction, CRUD, contexts, and SPARQL.

Evidence gates:

- Core CRUD, duplicates, invalid input, and named-graph behavior have tests.
- Public APIs build without Clippy warnings.
- Rust 1.87 and stable Rust are tested.
- Every exposed item has Rustdoc documentation.
- 0.1 inventory rows cite implementation and test locations.
- The temporary eager-query and direct-type-re-export decisions are documented.

Not in this milestone: polished parser/serializer facades, durable storage,
transactions, or a C ABI.

## 0.2 — RDF input and output

Outcome: match Redland parser and serializer workflows through safe,
stream-oriented APIs.

Deliverables:

- Safe `Parser` and `Serializer` facades.
- Reader, writer, string, file, and base-IRI entry points.
- Syntax discovery by name, MIME type, and file extension.
- Namespace configuration and serializer features.
- Turtle, N-Triples, N-Quads, TriG, and RDF/XML where supported by Oxigraph.
- Explicit parser source, graph-target, and blank-node-scope semantics.
- Bounded streaming paths that do not require loading an entire document.

Evidence gates:

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
- Query and update behavior is compared with native Redland fixtures.
- Iterator lifetimes do not require materializing full result sets.
- Ordering is asserted only where SPARQL guarantees it.
- Dataset/default-graph behavior has differential fixtures.
- Query error categories preserve parse versus evaluation failures.

Depends on: 0.2 dataset loading and result serialization.

## 0.4 — Storage and transactions

Outcome: cover durable models and Redland storage semantics with explicit
backend capabilities.

Deliverables:

- Stable memory and RocksDB storage constructors.
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
- RocksDB remains an opt-in feature and the no-default-features build passes.

Depends on: stable model, I/O, and update behavior from 0.1–0.3.

## 0.5 — Streams, utilities, and observability

Outcome: complete the non-query high-level Rust surface needed for Redland
workflow parity.

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

Deliverables:

- Finish the generated Redland function/type/enum inventory.
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

## 0.7 — C ABI preview

Outcome: run representative existing C consumers against an auditable Oxiland
compatibility library.

Deliverables:

- A separate `oxiland-capi` workspace crate.
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

Depends on: 0.6 safe API accounting and stable ownership semantics.

## 0.8 — C compatibility and ecosystem validation

Outcome: prove broad source and behavioral compatibility using real consumers.

Deliverables:

- Complete applicable C symbol implementations.
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

Depends on: a sanitizer-clean 0.7 ABI preview.

## 0.9 — Release candidate

Outcome: freeze and validate the design intended for 1.0.

Deliverables:

- API and ABI stabilization.
- Cross-platform packaging and installation documentation.
- Upgrade guide for Redland users.
- Security, fuzzing, interoperability, and performance hardening.
- Support, deprecation, MSRV, and vulnerability-response policies.
- Reproducible source archives and checksummed release artifacts.

Evidence gates:

- Rust public-API snapshots and C ABI snapshots are enforced in CI.
- The full conformance and differential matrix is green.
- Documentation includes complete examples for supported Redland workflows.
- Release candidates receive real downstream testing.
- Remaining deviations are enumerated and do not contradict the 1.0 claim.
- No release-blocking item remains in the risk register.
- A clean environment can install, link, execute, and uninstall every artifact.
- At least one release-candidate soak period completes without an ABI reset.

## 1.0 readiness

Version 1.0 is eligible only when both safe Rust accounting and the promised C
compatibility surface meet their published definitions. “Powered by Oxigraph”
does not imply that every historical Redland storage plug-in can be reproduced;
where exact implementation parity is impossible, 1.0 must provide compatible
observable behavior or an explicit, narrowly justified exclusion.

The release decision consumes the evidence defined above; elapsed time,
inventory percentages alone, or a green unit-test suite are insufficient.
