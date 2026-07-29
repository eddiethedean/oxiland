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

## Proposed decisions

### ADR-004 — Public RDF terms: re-export or wrapper

State: proposed  
Decision deadline: before 0.2 public facade expansion

Question: should Oxiland continue directly re-exporting Oxigraph RDF terms, or
introduce Oxiland-owned wrappers that can reproduce Redland-specific behavior?

Evaluation criteria:

- ability to express Redland node construction and introspection;
- API stability across Oxigraph upgrades;
- conversion and allocation overhead;
- blank-node and literal semantics;
- ergonomics for the broader Oxigraph ecosystem;
- impact on future C handles.

### ADR-005 — Streaming API shape

State: proposed  
Decision deadline: 0.1 completion

Question: should model, parser, and query streams use standard iterators,
lending iterators, callback visitors, or separate owned/borrowed adapters?

Evaluation criteria:

- no unbounded materialization;
- clear error propagation;
- model/query lifetime safety;
- cancellation and early termination;
- future C stream mapping.

### ADR-006 — Persistent storage compatibility boundary

State: proposed  
Decision deadline: before 0.4

Question: does Oxiland promise only logical dataset compatibility, or any
on-disk compatibility across Oxigraph/Oxiland versions?

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

