# SOLID boundaries

Status: implemented architecture note  
Scope: durable storage dispatch and the C model adapter

This note records the internal boundaries introduced during the 0.12
performance phase. The refactor preserves the public Rust API and exported C
ABI while making the two highest-coupling implementation areas easier to
change and test.

## Durable storage

`DurableStore` is a cloneable, type-erased facade over the sealed
`DurableStoreOps` interface. Backend adapters implement that interface and own
their engine-specific calls. Backend selection remains in `DurableStore::open`;
normal model operations no longer repeat a match over every compiled backend.

```text
Model
  -> DurableStore
       -> DurableStoreOps
            <- FjallStore adapter
            <- RedbStore adapter
            <- RocksDbStore adapter
            <- SqliteStore adapter
            <- LmdbStore adapter
```

This dependency direction applies the SOLID principles:

- **Single responsibility:** the facade controls construction and shared
  identity, while each adapter translates operations for one engine.
- **Open/closed:** adding a compiled backend adds a constructor branch and one
  adapter implementation instead of modifying every storage operation.
- **Liskov substitution:** each enabled adapter is exercised by the same
  backend conformance contract before it can stand behind `DurableStore`.
- **Interface segregation:** adapters implement only the persistence operations
  consumed by the model rather than exposing engine-specific APIs upstream.
- **Dependency inversion:** model code depends on the operation interface, not
  on concrete engine variants.

The interface remains sealed deliberately. It is an internal change boundary,
not a promise that arbitrary user-defined storage engines can satisfy the 1.0
durability contract.

To add a first-party durable backend:

1. implement the existing storage operations on its adapter type;
2. add the adapter through `impl_durable_adapter!`;
3. select it in `DurableStore::open` and recognize its on-disk format in
   `looks_like_store`;
4. run backend conformance tests with default, minimal, and all features.

## C model adapter

The C ABI still exports the same `librdf_model_*` entry points through the
model handle module, but their implementations are grouped by reason to
change:

| Module | Responsibility |
|---|---|
| `navigation` | projected node iteration and arc/source/target queries |
| `context` | named-graph operations and context serialization |
| `io` | parser/serializer selection and byte or string I/O |
| `feature` | model feature translation |
| `transaction` | transaction lifecycle entry points |
| `state` | cardinality-cache and transaction invariants |

Shared navigation helpers centralize projection, ownership, and membership
rules. Shared state objects make cache invalidation and transaction transitions
explicit instead of exposing sentinel fields to unrelated operations. The root
module retains handle construction, mutation, matching, and common FFI helpers.

This split applies single responsibility to the ABI adapter without moving RDF
semantics into the C layer. It also makes each concern independently testable
while keeping panic containment, pointer validation, and C ownership rules at
the boundary.

## Preserved constraints

- The safe Rust crate does not depend on the C ABI crate.
- No exported Rust or C API is removed or renamed.
- Backend capability and format commitments remain unchanged.
- C results retain their existing owned-versus-borrowed behavior.
- Unsupported operations continue to fail explicitly.

The verification baseline is workspace formatting and linting, the full Rust
test suite, C lifecycle tests, backend conformance tests, documentation checks,
and builds with default, no-default, and all features.
