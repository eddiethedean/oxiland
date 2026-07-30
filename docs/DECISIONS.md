# Architecture decision log

Status: active  
Format: lightweight architecture decision records (ADRs)

This log captures choices that constrain compatibility, public APIs, storage,
or the future C ABI. Proposed decisions remain open until their evidence and
tradeoffs are reviewed.

## Decision states

- `proposed`: under review and not safe to build upon.
- `accepted`: current project direction.
- `superseded`: replaced by a later decision.
- `rejected`: considered and intentionally not selected.

## Accepted decisions

### ADR-001 — Oxigraph is the RDF engine

State: accepted  
Milestone: project foundation

Context: Oxiland needs RDF term types, dataset storage, syntax processing, and
SPARQL without rebuilding mature standards implementations.

Decision: use a version-pinned Oxigraph dependency as the engine behind a
Redland-oriented safe facade.

Consequences:

- Oxigraph upgrades require compatibility and conformance verification.
- Semantic mismatches are handled by private Oxiland adapters.
- Oxiland does not promise every Oxigraph API as part of its stable API.

Revisit when: an applicable Redland behavior cannot be adapted without an
Oxigraph fork, or the dependency no longer meets platform/security needs.

### ADR-002 — Safe Rust and C ABI are separate crates

State: accepted  
Milestone: architecture baseline

Context: Redland's pointer ownership and callback conventions require `unsafe`
code, while the primary Rust API can remain safe.

Decision: keep the main `oxiland` crate free of unsafe code. Introduce
`oxiland-capi` no earlier than 0.7 as the only legacy ABI boundary.

Consequences:

- `oxiland` retains `#![forbid(unsafe_code)]`.
- C allocation, strings, opaque handles, and panic containment are audited
  independently.
- Safe API design is completed before ABI freezing.

Revisit when: a platform integration proves impossible without a narrowly
scoped safe-crate exception. Such an exception requires a superseding ADR.

### ADR-003 — Claims use independent compatibility levels

State: accepted  
Milestone: planning baseline

Context: workflow parity, safe API accounting, C source compatibility, ABI
compatibility, and behavioral parity require different evidence.

Decision: publish and track these claims separately. “100% parity” must name
its inventory, platform, features, and evidence revision.

Consequences:

- A single blended completion percentage is prohibited.
- Release notes identify the exact claim level reached.
- Exclusions cannot be hidden behind safe Rust replacements.

Revisit when: never; a replacement must preserve equally explicit claims.

### ADR-004 — Public RDF terms re-export Oxigraph types

State: accepted  
Milestone: 0.1

Context: Oxiland needs RDF terms immediately, while Redland-specific node
construction and introspection may later require wrappers.

Decision: re-export Oxigraph RDF term types from `oxiland::terms` for 0.1 and
provide thin helpers (`named_node`, `blank_node`) that map construction
failures into [`Error::InvalidRdf`]. Introduce owned wrappers only when a
verified Redland behavior cannot be expressed through Oxigraph types plus
adapters.

Alternatives:

- Wrap every term type now (higher conversion cost, earlier ABI handle design).
- Hide Oxigraph types entirely behind Oxiland-only constructors.

Consequences:

- Callers interoperate with the Oxigraph ecosystem without adapters.
- Public API snapshots include Oxigraph type names via re-exports.
- A later wrapper migration is a breaking change and must be gated by evidence.

Evidence: `src/lib.rs`, `tests/model.rs` invalid-input cases, API snapshot in
`api/oxiland-public-api.txt`.

Revisit when: a differential fixture requires Redland node behavior that
Oxigraph types cannot represent, or before expanding the C handle model in 0.7.

### ADR-005 — Model matching uses standard fallible iterators

State: accepted  
Milestone: 0.1

Context: Redland statement matching returns streams. Eager `Vec` collection
creates unbounded memory risk (R-007) and blocks early termination.

Decision: [`Model::find`] returns [`StatementMatches`], a standard
`Iterator<Item = Result<Quad>>` backed by an Oxigraph store snapshot. Parser
and query result streaming shapes remain open until 0.2/0.3; lending iterators
and callback visitors are deferred unless standard iterators prove insufficient.

Alternatives:

- Keep eager `Vec` with a documented removal milestone.
- Lending iterators or visitor callbacks for zero-copy access.

Consequences:

- Matching is lazy and supports early termination.
- Snapshot semantics mean results do not borrow the live model.
- Future C stream mapping can wrap the same iterator adapter pattern.

Evidence: `src/model.rs`,
`tests/model.rs::find_streams_without_full_materialization`.

Revisit when: zero-copy lending access is required, or query/parser streams in
0.2–0.5 need a shared streaming trait.

## Proposed decisions

### ADR-006 — Persistent storage compatibility boundary

State: proposed  
Decision deadline: before 0.4

Question: does Oxiland promise only logical dataset compatibility, or any
on-disk compatibility across Oxigraph/Oxiland versions for the fjall-backed
`Model::open` store?

Evaluation criteria:

- Oxigraph's storage guarantees;
- backup and migration feasibility;
- crash consistency;
- user expectations from Redland stores;
- release and support cost.

## ADR template

```markdown
### ADR-NNN — Title

State: proposed
Decision deadline: milestone or trigger

Context: why a durable decision is needed.

Decision: the selected direction.

Alternatives: meaningful options considered.

Consequences: compatibility, API, safety, performance, and operational effects.

Evidence: tests, prototypes, or source references.

Revisit when: concrete trigger.
```
