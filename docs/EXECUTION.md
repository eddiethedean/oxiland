# Execution plan

Status: active operating model  
Current milestone: 0.6 (0.5 complete)

This plan turns roadmap outcomes into reviewable work. It deliberately avoids
calendar estimates until the API inventory and differential harness reveal the
true compatibility surface.

The [project charter](CHARTER.md) constrains scope, the
[roadmap](ROADMAP.md) defines release outcomes, and
[milestone plans](milestones/0.3.md) define executable work packages. This document
owns the operating model and current work order.

## Workstreams

| Workstream | Responsibility | Primary evidence |
|---|---|---|
| Safe API | Rust facade, errors, lifetimes, capabilities | Rust tests and API snapshot |
| RDF/SPARQL | syntax and query behavior | W3C conformance |
| Compatibility | inventory, mappings, deviations | parity ledger and differential tests |
| Storage | persistence, transactions, migration | backend matrix |
| Python | Pythonic PyPI package over the safe facade | wheels, pytest, typing |
| C ABI | handles, symbols, headers, callbacks | ABI tests and sanitizers |
| Tooling | CLI, generators, packaging, CI | clean-install/downstream tests |
| Documentation | migration, examples, decisions | built docs and link checks |

A change may span workstreams, but it must have one primary acceptance path.

## Unit of work

Compatibility work should be sliced vertically. A complete slice includes:

1. one or more Redland inventory IDs;
2. a documented safe Rust mapping;
3. implementation through the public API;
4. positive, boundary, and failure tests;
5. differential evidence where native Redland behavior applies;
6. parity-ledger and user-documentation updates.

Infrastructure-only work may omit an inventory ID but must name the release gate
it enables. Code without mapping or evidence remains `implemented`, not
`verified`.

## Milestone workflow

### 1. Baseline

- Freeze canonical Redland headers, documentation, and oracle build metadata.
- Generate or refresh the milestone's inventory slice.
- Confirm Oxigraph version and relevant capabilities.
- Resolve blocking decisions and identify risks.

### 2. Design

- Write the Rust mapping and observable compatibility contract.
- Identify owned/borrowed lifetimes, streaming behavior, errors, and features.
- Define fixtures before finalizing the public API.
- Record decisions that constrain later ABI work.

### 3. Implement

- Land the smallest end-to-end public workflow.
- Keep backend-specific code behind private adapters.
- Add capability errors before accepting unsupported names or options.
- Avoid compatibility placeholders that always fail.

### 4. Verify

- Run local quality gates.
- Run applicable W3C and differential fixtures.
- Update evidence links and metrics.
- Test both default and feature-minimal configurations.

### 5. Release

- Check every roadmap evidence gate.
- Review deviations and risk register entries.
- Generate migration and release notes.
- Test packaged artifacts from a clean environment.
- Mark the milestone complete only after evidence is durable and linked.

## Completed 0.1 backlog

| Priority | Deliverable | Evidence |
|---:|---|---|
| P0 | Curated Redland API inventory for 0.1 | `compatibility/inventory/…0.1.json`, `scripts/check-inventory.py` |
| P0 | Model/context CRUD semantics | `tests/model.rs` named-graph isolation |
| P0 | Direct Oxigraph term re-exports | ADR-004 |
| P0 | Streaming `Model::find` | ADR-005, `StatementMatches` |
| P1 | Stable error categories | `SparqlParse` / `SparqlEvaluation` / `InvalidRdf` / `Unsupported` tests |
| P1 | Thread-safety and clone semantics | Rustdoc + Send/Sync and clone-sharing tests |
| P1 | CI for Rust 1.87 and stable | `.github/workflows/ci.yml` |
| P1 | Public-API snapshot tooling | `api/oxiland-public-api.txt`, `scripts/generate-public-api.sh` |
| P2 | Runnable examples and doctests | examples in CI; crate doctests |
| P2 | 0.1 compatibility report | `docs/reports/0.1.md` |

## Completed 0.2 backlog

The detailed acceptance criteria and dependency map live in the
[0.2 milestone plan](milestones/0.2.md).

| Order | Work package | State | Completion evidence |
|---:|---|---|---|
| 1 | WP-02-00 baseline and I/O inventory | `complete` | baseline README, format matrix, 0.2 inventory, oracle smoke |
| 2 | WP-02-01 design spikes and decisions | `complete` | ADR-007/008, `docs/design/0.2-io-api.md` |
| 3 | WP-02-02 format and capability layer | `complete` | `Syntax` + `tests/io.rs` lookup coverage |
| 4a | WP-02-03 streaming parser | `complete` | `Parser`, early-stop and parse-error tests |
| 4b | WP-02-05 streaming serializer | `complete` | `Serializer`, round-trip and dataset tests |
| 5 | WP-02-04 model loading and file input | `complete` | progressive/collecting load + path helpers |
| 6 | WP-02-06 conformance and differential evidence | `complete` | `tests/conformance.rs`, harness scripts, CI job |
| 7 | WP-02-07 documentation and release | `complete` | `docs/reports/0.2.md`, examples, parity/roadmap updates |

### Immediate next actions

1. Begin 0.6 safe-API accounting inventory expansion.
2. Expand SPARQL differential fixtures beyond the facade smoke harness.
3. Keep format v1 / ADR-006 migration notes current as later milestones land.

## Completed 0.5 backlog

| Priority | Deliverable | Notes |
|---:|---|---|
| P0 | ADR-013–ADR-016 | Streams policy, logging, digests, hash/list |
| P0 | Utility module | URI/file/Unicode/digest/namespace/vocab |
| P0 | World logging | Handlers + optional `tracing` |
| P1 | Inventory + digest smoke | `redland-1.0.17-oxiland-0.5.json` |
| P2 | Migration example | `examples/std_replacements.rs` |

## Completed 0.4 backlog

| Priority | Deliverable | Notes |
|---:|---|---|
| P0 | ADR-006 storage compatibility boundary | Accepted; format v1 |
| P0 | Transaction / sync / clear APIs | `Model::transaction`, sync, clear |
| P1 | Storage capability reporting | `StorageCapabilities` + legacy disposition |
| P2 | Import/export for archival | N-Quads helpers + transactional load |

## Current 0.6 backlog

See the [roadmap 0.6 section](ROADMAP.md). Work-package details expand as design
spikes land.

## Completed 0.3 backlog

| Priority | Deliverable | Notes |
|---:|---|---|
| P0 | Query/result inventory slice | `redland-1.0.17-oxiland-0.3.json` |
| P0 | Streaming SELECT/bindings adapters | `Query` + `QueryResults` |
| P0 | SPARQL Update entry points | `Update` + Fjall resync |
| P1 | CONSTRUCT/DESCRIBE result forms | `tests/query.rs` |
| P1 | Limit/offset/base IRI configuration | Algebra `Slice` (ADR-009) |
| P2 | Result serialization formats | `ResultsFormat` XML/JSON/CSV/TSV |

## Definition of ready

Work is ready when:

- the target inventory entries or infrastructure gate are named;
- observable behavior and non-goals are clear;
- blocking architecture decisions are resolved;
- test data and oracle requirements are available;
- no unreviewed security or destructive-storage assumption is required.
- the work package's dependencies are complete or an explicit parallel-safe
  boundary is documented.

## Definition of done

Work is done when:

- implementation and public documentation agree;
- applicable tests pass in required feature configurations;
- inventory state and parity ledger are updated;
- compatibility differences are recorded, not hidden;
- no temporary eager allocation, panic path, or unsafe assumption is left
  without an owner and removal/review milestone.
- the work package status and evidence links are updated in the same change.

## Planning cadence

- Update the active work-package table when a package starts, blocks, or
  completes.
- Review the current milestone after each completed vertical slice.
- Review dependency and Oxigraph upgrade decisions before each minor release.
- Review accepted deviations and high risks at every milestone boundary.
- Review 1.0 scope after 0.6, after 0.7 (Python), and again after 0.9; do not
  defer incompatible safe-API corrections until 0.10.

## Progress reporting

Report counts separately:

- inventory entries by state;
- roadmap evidence gates passed/total;
- W3C tests passed/failed/skipped;
- differential fixtures passed/failed/skipped;
- open deviations by severity;
- blocking risks and decisions;
- supported downstream consumers.

Progress reports link to evidence revisions. Percent-complete estimates without
a denominator are not project status.

## Handling blocked or deferred work

A blocked package records the blocking decision, risk, or external dependency
and the next action that could unblock it. A deferral records:

- user-visible impact and workaround;
- destination milestone;
- owner workstream;
- inventory IDs or release gates affected;
- whether the current release outcome remains truthful.

Deferral is not completion. If a deferred item is necessary for the milestone
outcome, the milestone remains blocked or its scope changes through the project
charter and roadmap change-control process.
