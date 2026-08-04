# Architecture

Status: active design baseline  
Current implementation: Cargo workspace — library crate `oxiland`,
`crates/oxiland-cli` (0.6+), `crates/oxiland-capi` (0.8+ C ABI with 0.11
demonstrated packaging retained at tip 0.12), and `python/` PyPI package
(0.7+, not a workspace member). Oxigraph 0.5.9 remains the RDF engine.  
`fuzz/` holds parser/FFI/lifecycle targets.  
Next review gate: before 1.0 readiness (suite-wide performance claim and final
contract freeze)

This document specifies the implemented dependency direction, ownership model,
and safety boundaries. It is a design contract rather than an item-level API
reference.

## Design principles

These principles refine the [project charter](CHARTER.md); the charter wins if
the documents ever conflict.

1. Safe Rust is the primary implementation surface.
2. Oxigraph owns RDF representation, parsing, serialization, storage, and
   SPARQL execution wherever its semantics satisfy the compatibility contract.
3. Redland compatibility belongs in adapters, not in Oxigraph forks.
4. Legacy C ownership rules are isolated from the safe API.
5. Unsupported behavior fails explicitly; it is never silently approximated.
6. Observable compatibility wins over matching Redland's internal design.
7. Expensive behavior must be visible in API names, types, or documentation.
8. Public facades should be mock-free and testable against both Oxigraph and
   native Redland fixtures.

## System boundaries

```text
Rust callers ──> oxiland safe facade ──> Oxigraph
                       │
Python ──> oxiland (PyPI, 0.7+) ───────┤
                       │
C callers ──> oxiland-capi (0.8+) ─────┘
                       │
              allocation/callback shim

Conformance manifests ─┐
Redland oracle runner ─┼─> shared normalized fixtures ─> parity evidence
Downstream consumers ──┘
```

Dependency arrows point inward toward the safe facade and Oxigraph. The safe
crate never depends on the C ABI crate, the Python package, native Redland,
test harnesses, or the CLI. The Python package binds the safe Rust crate
directly; it is not layered on `oxiland-capi`.

## Repository and planned workspace

```text
oxiland/
├── src/                 Safe Rust facade
├── tests/               Rust integration tests
├── docs/                User, evaluator, contributor, and archive documentation
├── python/              Pythonic PyPI package (`oxiland`, 0.7+)
├── crates/
│   ├── oxiland-capi/    C ABI and opaque handle management (0.8+)
│   └── oxiland-cli/     rdfproc-shaped workflows (0.6+)
├── compatibility/
│   ├── inventory/       Generated Redland API manifests
│   ├── fixtures/        Shared behavioral fixtures
│   └── harness/         Native Redland differential runner
└── fuzz/                Parser, FFI, and lifecycle fuzz targets
```

Implemented through tip 0.12: the root Rust library, `crates/oxiland-cli`,
`crates/oxiland-capi` (0.11 demonstrated packaging retained), `python/`,
`fuzz/`, tests, documentation, compatibility inventory/fixtures/harness,
sealed durable storage adapters, and release automation. Remaining C behavioral
gaps are documented in user limitations—not silent expansions of the allowlist.

## Component responsibilities

| Component | Owns | Must not own |
|---|---|---|
| `terms` | RDF type exports and compatibility constructors | storage or parsing |
| `model` | datasets, contexts, transactions, streams, and backend-independent persistence policy | syntax detection or native engine calls |
| `storage` | backend identities, open options, capabilities, and the sealed durable adapter boundary | RDF/SPARQL semantics or silent backend substitution |
| `io` | parser/serializer configuration and byte I/O | persistent-store policy |
| `query` | query/update configuration and result adapters | C allocation |
| `world` | factories, shared features, and logging hooks | global mutable runtime |
| `utility` | URI, digest, file, vocabulary, and Unicode helpers | RDF engine logic |
| `oxiland-capi` | opaque handles and C ownership translation | RDF semantics |
| `python/` (PyPI) | idiomatic Python API over the safe facade | C ABI ownership; 1:1 Rust mirroring |
| `oxiland-cli` | command parsing and human-facing output | reusable domain behavior |

The facade should use owned and borrowed variants consistently. Fallible
construction returns `Result`; iteration exposes fallible iterators instead of
sentinel pointers.

## Data and lifetime model

- Oxigraph RDF terms are the canonical in-process representation unless an
  accepted decision record establishes a wrapper requirement.
- Models own cloneable Oxigraph store handles; cloning a model does not clone
  its dataset.
- Borrowed results may not outlive their model or query execution context.
- Iterator APIs should stream. `Model::find` returns `StatementMatches`
  (ADR-005). Parser (0.2) and query (0.3) streams follow the same fallible
  iterator policy documented in 0.5 (ADR-013).
- Blank-node identity is scoped by the parser or dataset operation that creates
  it; adapters must not derive identity from labels alone.
- User-provided callbacks are invoked outside internal locks whenever possible.

## Error and capability model

Errors should retain the subsystem and underlying cause while presenting stable
public categories: invalid RDF, parsing, serialization, query parse, query
evaluation, storage, I/O, unsupported capability, and C-boundary failure.

Backend- or version-specific behavior is queried through typed capabilities.
Feature URI support may adapt to those capabilities, but unknown feature URIs
must not be reported as successful no-ops.

## Concurrency and cancellation

Public types should be `Send` and `Sync` only when their Oxigraph-backed
behavior and callback contents make that truthful. Thread-safety is asserted in
tests and, for the C API, in a published per-handle matrix.

Long-running parse, query, update, and bulk-load operations need an explicit
cancellation policy. As of 0.3, `Query` and `Update` accept an Oxigraph
`CancellationToken` (ADR-012). Wall-clock timeouts are caller-driven by
cancelling the token from another thread. The API must not imply reliable
interruption where none exists.

## C ABI boundary

The C layer is a separate crate (`oxiland-capi`) allowed to contain narrowly
reviewed `unsafe` code. Its handles own or reference safe Oxiland objects. Each
handle type must define:

- allocation and destruction functions;
- null handling;
- clone/reference behavior;
- callback lifetime and thread rules;
- panic containment;
- error and logging translation.

No Rust panic may cross the C boundary. C strings and buffers must have one
documented allocator and matching free operation.

Additional invariants:

- every entry point validates nullability before dereferencing;
- opaque handles carry a type discriminator or equivalent misuse defense;
- destruction is idempotent only where Redland promises it;
- callbacks document re-entry, concurrency, and borrowed-pointer duration;
- thread-local “last error” state is used only if required by the mapped API;
- `catch_unwind` is a containment boundary, not a substitute for invariants.

## Compatibility mapping strategy

Redland concepts fall into three groups:

- Direct mappings: nodes, triples, models, contexts, parsers, serializers, and
  SPARQL.
- Semantic adapters: world factories, feature URIs, streams, storage options,
  logging, and result formatting.
- Rust replacements: manual allocation, generic lists, generic hashes, and
  other mechanisms already supplied more safely by Rust.

Rust replacements still require inventory entries and migration examples; they
are not omitted from parity accounting.

## Extension points

Storage, parser, serializer, query-language, and digest factories are
compatibility-sensitive extension points. Before exposing a public registration
API, decide whether it can be soundly represented as:

- a Rust trait with owned registrations;
- a finite Oxigraph-backed capability table; or
- a C-only callback adapter.

Factory names are data in Redland programs. Unknown names must produce an
explicit error and preserved diagnostic context.

Durable storage follows the staged
[backend expansion plan](design/storage-backend-expansion.md): extract a sealed
adapter and conformance harness in 0.8, add optional first-party engines in 0.9,
then decide whether a public user-supplied backend trait is supportable before
0.10. This revisits ADR-006 without weakening its Fjall format-v1 promise or
ADR-018's rejection of arbitrary native plug-in registration.

The implemented internal dependency direction and the responsibility split in
the C model adapter are recorded in the
[SOLID boundaries note](design/solid-boundaries.md).

## Dependency policy

Oxigraph is pinned intentionally within each Oxiland release. Upgrades require
the full conformance suite because RDF syntax, storage, and SPARQL changes can
affect observable compatibility. Additional dependencies should be small,
maintained, and justified by a Redland parity requirement.

Dependency review records:

- license and minimum supported Rust version;
- default features and native build requirements;
- maintenance and security posture;
- effect on WebAssembly and pure-Rust fjall builds;
- whether it crosses the safe/C boundary.

## Architecture decision records

Decisions that constrain later compatibility work are recorded in
[`DECISIONS.md`](DECISIONS.md). A decision record includes context, choice,
alternatives, compatibility impact, and a revisit trigger. Pull requests should
not reverse an accepted decision only through code changes.

## Open architecture questions

- Which C handles need reference counting to reproduce observed aliasing?
- Can the sealed durable-store adapter become a public custom-backend trait
  without freezing engine-specific lifetimes, transactions, or unsafe open
  requirements into the 1.0 facade (proposed ADR-022)?

Resolved recently: Redland factory registration disposition for safe Rust
(ADR-018); `oxiland-cli` rdfproc workflows (ADR-019); naming freeze intent
(ADR-020); header-derived inventory generation (ADR-021); sealed durable
adapter and backend matrix (ADR-022); C ABI ownership/panic/allocator
(ADR-023); 0.5 stream/utility surface (ADR-013–ADR-016); query cancellation
via Oxigraph `CancellationToken` (ADR-012).

Term re-exports are governed by ADR-004 and revisited only on its evidence
trigger. The remaining questions are decision candidates, not implicit TODOs.

## Reading this architecture

- Public Rust signatures and item semantics: [docs.rs](https://docs.rs/oxiland)
- Python contract: [Python API](users/python-api.md)
- Storage deployment: [Rust](users/rust-production.md) and
  [Python](users/python-production.md) production guides
- Current verified claims: [parity ledger](parity.md)
- Durable design rationale: [decision log](DECISIONS.md)
- Future sequencing: [roadmap](ROADMAP.md)
