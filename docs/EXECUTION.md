# Execution plan

Status: active operating model  
Current milestone: 0.2 (0.1 complete)

This plan turns roadmap outcomes into reviewable work. It deliberately avoids
calendar estimates until the API inventory and differential harness reveal the
true compatibility surface.

## Workstreams

| Workstream | Responsibility | Primary evidence |
|---|---|---|
| Safe API | Rust facade, errors, lifetimes, capabilities | Rust tests and API snapshot |
| RDF/SPARQL | syntax and query behavior | W3C conformance |
| Compatibility | inventory, mappings, deviations | parity ledger and differential tests |
| Storage | persistence, transactions, migration | backend matrix |
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

## Current 0.2 backlog

Priority is ordered; a lower item should not force the public shape of an
unresolved higher item.

| Priority | Deliverable | Completion evidence |
|---:|---|---|
| P0 | Safe `Parser` / `Serializer` facades | public API + round-trip tests |
| P0 | Syntax discovery by name, MIME, extension | lookup tests and unsupported errors |
| P1 | Reader/writer/string/file/base-IRI entry points | integration tests |
| P1 | Bounded streaming parse path | large-input memory test |
| P2 | Document supported vs unsupported Redland syntax names | compatibility notes |

## Definition of ready

Work is ready when:

- the target inventory entries or infrastructure gate are named;
- observable behavior and non-goals are clear;
- blocking architecture decisions are resolved;
- test data and oracle requirements are available;
- no unreviewed security or destructive-storage assumption is required.

## Definition of done

Work is done when:

- implementation and public documentation agree;
- applicable tests pass in required feature configurations;
- inventory state and parity ledger are updated;
- compatibility differences are recorded, not hidden;
- no temporary eager allocation, panic path, or unsafe assumption is left
  without an owner and removal/review milestone.

## Planning cadence

- Review the current milestone after each completed vertical slice.
- Review dependency and Oxigraph upgrade decisions before each minor release.
- Review accepted deviations and high risks at every milestone boundary.
- Review 1.0 scope after 0.6 and again after 0.8; do not defer incompatible
  safe-API corrections until 0.9.

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

