# Project charter

Status: active product contract  
Applies to: all Oxiland design, implementation, compatibility, and release work  
Review: at every minor milestone and before changing the 1.0 promise

## Mission

Oxiland provides a safe, maintained Rust successor to Redland `librdf`, using
Oxigraph for RDF and SPARQL standards machinery while preserving the Redland
workflows that real applications depend on.

The project succeeds by making compatibility precise and testable. It does not
try to reproduce Redland's internal architecture, expose all of Oxigraph, or
describe an aspiration as shipped behavior.

## People and jobs to be done

Oxiland serves three audiences, in priority order:

1. **Rust application authors** who want Redland-shaped RDF concepts without C
   ownership and memory-management hazards.
2. **Maintainers migrating Redland software** who need an explicit mapping,
   known deviations, and evidence for each supported workflow.
3. **Existing C consumers** who eventually need a separately audited
   compatibility library and a realistic path away from a legacy native
   dependency.

A **Python package** (ships 0.7 as `oxiland` on PyPI) extends the first audience
to Python application authors with idiomatic interfaces over the same safe
facade. It is not a substitute for the C ABI track.

The safe Rust API is the product foundation. The Python package and the C layer
are adapters over that foundation, not independent RDF engines.

## Product promises

Oxiland aims to provide:

- safe, documented Rust APIs for applicable Redland concepts;
- standards-correct RDF and SPARQL behavior through Oxigraph;
- explicit capabilities and useful typed failures for unsupported behavior;
- streaming interfaces for potentially unbounded data;
- evidence-scoped compatibility reports that can be reproduced;
- a migration path for selected C consumers after the Rust semantics stabilize.

During 0.x, public Rust APIs and persistent formats may change. Such changes
must be intentional, documented, and accompanied by migration guidance.
Behavioral changes in RDF, SPARQL, storage, or ownership semantics must never be
silent.

## Scope

In scope:

- public Redland `librdf` 1.0.17 concepts and the manual labeled 1.0.18;
- Raptor and Rasqal behavior when observable through a public `librdf`
  workflow;
- RDF terms, datasets, contexts, parsing, serialization, SPARQL, storage,
  streams, utilities, logging, and the `rdfproc` class of workflows;
- safe Rust migration APIs;
- a Pythonic PyPI package over the safe facade (ships 0.7; not a 1:1 Rust mirror);
- a later C source/ABI compatibility layer for an explicitly published
  platform and symbol matrix;
- tooling needed to inventory, compare, package, and verify those claims.

## Non-goals

Unless a later decision explicitly expands the charter, Oxiland does not aim
to:

- reimplement all independent Raptor or Rasqal APIs;
- clone Redland internals or preserve implementation-specific data structures;
- expose every Oxigraph API through the Oxiland stability promise;
- silently map legacy storage backend names to unrelated technologies;
- guarantee 0.x on-disk compatibility without an accepted storage decision;
- promise network protocol, server, or distributed-database functionality;
- preserve undefined behavior, memory bugs, or unsafe invalid-input behavior;
- claim support based only on code presence, an inventory percentage, or
  Oxigraph's upstream test results.

## Design values

When goals compete, use this order:

1. Memory safety and data integrity.
2. Honest, observable compatibility.
3. RDF/SPARQL correctness.
4. A coherent Rust API.
5. Diagnosability and operational clarity.
6. Bounded resource use.
7. Performance.
8. Breadth of legacy coverage.

This order does not excuse avoidable performance problems. It makes the
decision rule explicit when exact legacy behavior would compromise safety or
correctness.

## Compatibility boundary

The project tracks independent compatibility claims:

- concept parity;
- complete safe-API accounting;
- safe behavioral compatibility;
- C source compatibility;
- C ABI compatibility;
- downstream compatibility.

Each claim is scoped by subsystem, platform, enabled features, and evidence
revision. The normative definitions are in
[`COMPATIBILITY.md`](COMPATIBILITY.md); current evidence is in
[`parity ledger`](parity.md). “Redland compatible” without that scope is not a
release claim.

## Foundation invariants

The following constraints require a superseding architecture decision:

- the primary `oxiland` crate forbids unsafe code;
- C ownership and callbacks live in a separate crate;
- Oxigraph is version-pinned within a release;
- unknown backends, formats, features, and factories fail explicitly;
- compatibility-sensitive work maps to inventory entries or a named release
  gate;
- public unbounded workflows stream unless a documented constraint proves
  bounded materialization;
- planned components are labeled as planned and are not presented as available;
- releases are evidence-gated, not date- or percentage-gated.

## Success measures

Progress is reported as separate measures, never a blended score:

| Measure | What is counted | 1.0 expectation |
|---|---|---|
| Inventory accounting | public Redland items by state | no unclassified items |
| Rust behavior | mapped workflows with public-API tests | all promised workflows pass |
| Standards conformance | applicable W3C cases | published pass/skip/fail totals |
| Differential behavior | normalized Redland fixtures | no unexplained differences |
| Safety | fuzzing, sanitizers, lifecycle tests | no release-blocking findings |
| Portability | supported target matrix | all published targets green |
| Downstream proof | selected unchanged consumers | published matrix passes |
| Operability | install, migrate, back up, recover | documented and tested |

Every percentage includes a numerator, denominator, skipped count, and suite or
inventory revision.

## 1.0 definition

Version 1.0 means:

- the safe Rust surface promised by the project is reviewed and documented;
- every public Redland item is mapped, intentionally replaced, or explicitly
  excluded with impact and workaround;
- the published behavioral, source, and ABI matrices meet their stated gates;
- API, ABI, persistence, MSRV, support, and deprecation policies are published;
- clean installation and selected downstream workflows work from release
  artifacts;
- no open risk contradicts the release claims.

It does not necessarily mean every historical Redland storage plug-in or every
platform is supported. The supported boundary must be explicit enough that a
user can decide whether migration is safe before adopting Oxiland.

## Change control

A change needs a decision record when it:

- alters a foundation invariant;
- changes public ownership, lifetime, streaming, persistence, or error
  semantics;
- expands or narrows a compatibility claim;
- introduces an extension or registration mechanism;
- commits the project to a persistent format, ABI, or new supported platform.

Small implementation choices do not need ADRs. They still need tests and
documentation if externally observable.

The roadmap owns release outcomes, milestone plans own executable work,
the parity ledger owns current verified status, and reports preserve historical
evidence. The [planning index](index.md) defines the complete authority order.
