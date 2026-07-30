# Architecture plan

Status: active design baseline  
Current implementation: single `oxiland` crate on Oxigraph 0.5.9  
Next review gate: before expanding storage/transaction contracts in 0.4

This document specifies dependency direction and safety boundaries. It does not
claim that planned crates or modules already exist.

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

## Intended workspace

```text
oxiland/
├── src/                 Safe Rust facade
├── tests/               Rust integration tests
├── docs/                Plans and compatibility documentation
├── python/              Pythonic PyPI package (0.7+; name TBD)
├── crates/
│   ├── oxiland-capi/    C ABI and opaque handle management (0.8+)
│   └── oxiland-cli/     rdfproc-compatible workflows (0.6+)
├── compatibility/
│   ├── inventory/       Generated Redland API manifests
│   ├── fixtures/        Shared behavioral fixtures
│   └── harness/         Native Redland differential runner
└── fuzz/                Parser, FFI, and lifecycle fuzz targets
```

The additional crates and directories are planned, not present yet.

## Component responsibilities

| Component | Owns | Must not own |
|---|---|---|
| `terms` | RDF type exports and compatibility constructors | storage or parsing |
| `model` | datasets, contexts, storage, transactions, streams | syntax detection |
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
  (ADR-005). Parser and query streams follow in later milestones.
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

The future C layer is a separate crate allowed to contain narrowly reviewed
`unsafe` code. Its handles own or reference safe Oxiland objects. Each handle
type must define:

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

- What transaction abstraction can cover memory and fjall consistently?
- Can query cancellation be implemented without modifying Oxigraph?
  (Resolved in 0.3 via Oxigraph `CancellationToken`; see ADR-012.)
- Which Redland factory registrations are safe and useful in Rust?
- Which C handles need reference counting to reproduce observed aliasing?

Term re-exports are governed by ADR-004 and revisited only on its evidence
trigger. The remaining questions are decision candidates, not implicit TODOs.
