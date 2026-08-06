# Architecture decision log

Status: active  
Format: lightweight architecture decision records (ADRs)

This log captures choices that constrain compatibility, public APIs, storage,
or the future C ABI. Proposed decisions remain open until their evidence and
tradeoffs are reviewed.

## Index

| ADR | Title |
|---|---|
| [ADR-001](#adr-001-oxigraph-is-the-rdf-engine) | Oxigraph is the RDF engine |
| [ADR-002](#adr-002-safe-rust-and-c-abi-are-separate-crates) | Safe Rust and C ABI are separate crates |
| [ADR-003](#adr-003-claims-use-independent-compatibility-levels) | Claims use independent compatibility levels |
| [ADR-004](#adr-004-public-rdf-terms-re-export-oxigraph-types) | Public RDF terms re-export Oxigraph types |
| [ADR-005](#adr-005-model-matching-uses-standard-fallible-iterators) | Model matching uses standard fallible iterators |
| [ADR-006](#adr-006-persistent-storage-compatibility-boundary) | Persistent storage compatibility boundary |
| [ADR-007](#adr-007-parser-output-and-model-load-failure-semantics) | Parser output and model-load failure semantics |
| [ADR-008](#adr-008-built-in-rdf-format-identity-and-discovery) | Built-in RDF format identity and discovery |
| [ADR-009](#adr-009-queryupdate-builders-dataset-unbound-limitoffset) | Query/Update builders, dataset, unbound, limit/offset |
| [ADR-010](#adr-010-streaming-query-result-adapters) | Streaming query result adapters |
| [ADR-011](#adr-011-sparql-results-serialization-formats) | SPARQL results serialization formats |
| [ADR-012](#adr-012-query-cancellation-policy) | Query cancellation policy |
| [ADR-013](#adr-013-shared-streaming-policy-without-a-unifying-trait) | Shared streaming policy without a unifying trait |
| [ADR-014](#adr-014-world-logging-facade-and-optional-tracing) | World logging facade and optional tracing |
| [ADR-015](#adr-015-closed-digest-algorithm-set) | Closed digest algorithm set |
| [ADR-016](#adr-016-hashes-and-lists-map-to-standard-rust-collections) | Hashes and lists map to standard Rust collections |
| [ADR-017](#adr-017-python-package-is-pythonic-not-a-thin-rust-mirror) | Python package is Pythonic, not a thin Rust mirror |
| [ADR-018](#adr-018-factory-registration-disposition-for-safe-rust) | Factory registration disposition for safe Rust |
| [ADR-019](#adr-019-oxiland-cli-rdfproc-workflow-surface) | `oxiland-cli` rdfproc workflow surface |
| [ADR-020](#adr-020-10-naming-and-module-freeze-intent) | 1.0 naming and module freeze intent |
| [ADR-021](#adr-021-header-derived-inventory-generation) | Header-derived inventory generation |
| [ADR-022](#adr-022-sealed-durable-adapter-and-optional-backend-matrix) | Sealed durable adapter and optional backend matrix |
| [ADR-023](#adr-023-c-abi-ownership-panic-and-allocator-contract) | C ABI ownership, panic, and allocator contract |
| [ADR-024](#adr-024-freeze-first-party-storage-and-keep-the-adapter-sealed) | Freeze first-party storage and keep the adapter sealed |
| [ADR-025](#adr-025-baseline-factory-registration-for-010-parity) | Baseline factory registration for 0.10 parity |
| [ADR-026](#adr-026-raptor-and-rasqal-world-bridges-for-librdf) | Raptor and Rasqal world bridges for `librdf` |
| [ADR-027](#adr-027-keep-heedbincode-for-optional-lmdb-in-010) | Keep heed/bincode for optional LMDB in 0.10 |
| [ADR-028](#adr-028-012-competitive-parity-performance-gate) | 0.12 competitive-parity performance gate |
| [ADR-029](#adr-029-013-suite-wide-faster-than-redland-gate) | 0.13 suite-wide faster-than-Redland gate |

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
`oxiland-capi` no earlier than 0.8 as the only legacy C ABI boundary. The
Python package (ships 0.7) binds the safe Rust crate directly and is not layered
on `oxiland-capi`.

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
Oxigraph types cannot represent, or before expanding the C handle model in 0.8.

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

Revisit when: zero-copy lending access is required, or C ABI stream handles
need a shared adapter (0.8). 0.5 documented a shared fallible-iterator policy
without a unifying trait (ADR-013).

### ADR-007 — Parser output and model-load failure semantics

State: accepted  
Milestone: 0.2

Context: a streaming parser can yield valid statements before encountering
malformed input. Loading the same source into a model could therefore leave
partial data unless the API stages input or uses a transaction. The 0.2 model
does not yet have the transaction abstraction planned for 0.4.

Decision:

- The public streaming parser exposes `Iterator<Item = Result<Quad>>` with
  explicit partial progress, wrapping Oxigraph `RdfParser` (never
  `Store::load_from_*` for the stream path).
- Facade parses always enable `rename_blank_nodes()`.
- Default model convenience methods (`Parser::load_into`) insert progressively;
  on parse, I/O, or insert failure after progress, already-inserted quads remain
  and the error documents that a partial load occurred.
- An explicitly named collecting path (`Parser::load_collecting`) buffers the
  complete successful quad set and inserts only after parse success. If a later
  insert fails, quads newly inserted by that call are removed best-effort.
- As of 0.4, `Parser::load_transactional` / `load_path_transactional` parse
  fully then insert inside `Model::transaction` (durable sync on Fjall commit).
  Progressive and collecting paths remain available.

Alternatives:

- Omit model-load helpers until transactions exist.
- Always buffer (unbounded memory for large files).
- Claim atomic progressive load without transactions (dishonest on Fjall).

Consequences:

- Callers choose streaming honesty versus buffered all-or-nothing by API name.
- R-017 is mitigated by documentation and error text rather than false
  atomicity.
- 0.4 added transactional load without breaking the streaming core.

Evidence: `src/io/parser.rs`,
`tests/io.rs::progressive_load_leaves_partial_data_on_failure`,
`tests/io.rs::progressive_load_annotates_partial_data_on_io_failure`,
`tests/io.rs::collecting_load_is_all_or_nothing`,
`tests/storage.rs::transactional_load_is_atomic_on_parse_failure`,
[docs/design/0.2-io-api.md](design/0.2-io-api.md).

Revisit when: differential fixtures require Redland callback-equivalent
atomicity beyond `load_transactional`.

### ADR-008 — Built-in RDF format identity and discovery

State: accepted  
Milestone: 0.2

Context: Redland selects parser and serializer factories through names, MIME
types, and other aliases. Oxigraph exposes a finite set of format values.
Treating arbitrary strings as formats would make capability reporting unstable
and could prematurely commit Oxiland to public custom registration.

Decision: expose a closed `Syntax` enum for Turtle, N-Triples, N-Quads, TriG,
and RDF/XML, backed by a curated alias table for Redland names, media types,
and extensions. Unknown, ambiguous, or deferred aliases return
`Error::Unsupported`. N3 and JSON-LD are not advertised in 0.2. Custom factory
registration is deferred. Oxigraph primitives remain under
`oxiland::io::primitives`.

Alternatives:

- String-keyed public registry from day one.
- Re-export Oxigraph `RdfFormat` as the public identity.

Consequences:

- Capability queries and constructors share one table (R-018).
- Adding a syntax is an intentional SemVer-visible change.
- Redland `guess`/content sniffing remains unsupported.

Evidence: `src/io/format.rs`,
`compatibility/baseline/format-matrix.json`,
`tests/io.rs::syntax_lookup_covers_names_media_types_and_extensions`.

Revisit when: custom factories are required for C consumers, or JSON-LD /
true N3 must be advertised.

### ADR-009 — Query/Update builders, dataset, unbound, limit/offset

State: accepted  
Milestone: 0.3

Context: Redland exposes query configuration (base, limit, offset, dataset)
separately from result iteration. Oxigraph 0.5.9 provides `SparqlEvaluator`,
`QueryDatasetSpecification`, and spargebra algebra, but not Redland-shaped
builders.

Decision:

- Owned `Query` and `Update` builders configure base IRI, prefixes, dataset,
  and cancellation before execution.
- Unbound solution bindings are `None` via `QuerySolution::get` (name or
  position).
- API `limit`/`offset` apply `GraphPattern::Slice` after spargebra parse for
  SELECT/CONSTRUCT/DESCRIBE; ASK rejects API slice with `Unsupported`.
- Dataset defaults map to Oxigraph `dataset_mut` /
  `using_datasets_mut` helpers.

Alternatives: string-rewrite LIMIT/OFFSET; force SPARQL-text-only limits.

Consequences: algebra dependency on pinned `spargebra = "=0.4.6"`; clear error
when slicing ASK.

Evidence: `src/query.rs`, `docs/design/0.3-query-api.md`, `tests/query.rs`.

Revisit when: Oxigraph gains first-class prepared-query limit APIs.

### ADR-010 — Streaming query result adapters

State: accepted  
Milestone: 0.3

Context: ADR-005 left query streaming open. Callers must not be forced to
collect full solution or graph result sets.

Decision: own a thin `QueryResults` enum that wraps Oxigraph's streaming
variants (`Boolean`, `Solutions`, `Graph`) so the facade can provide a useful
[`Debug`] without draining iterators. Document early-stop by dropping iterators.
`oxiland::sparql` remains an escape hatch (including Oxigraph's own
`QueryResults`); inventory cites the owned `Query` / `Update` / `ResultsFormat`
surface.

Alternatives: wrap every row in owned Oxiland enums; lending iterators.

Consequences: lifetimes borrow the model/store snapshot semantics of Oxigraph;
errors inside iterators map through `SparqlEvaluation` at the call site.

Evidence: `tests/query.rs` early-stop cases; `docs/design/0.3-query-api.md`.

Revisit when: lending iterators or C ABI stream handles require a shared trait
beyond the 0.5 fallible-iterator policy (ADR-013).

### ADR-011 — SPARQL results serialization formats

State: accepted  
Milestone: 0.3

Context: Redland serializes query results in several formats. Oxigraph exposes
XML/JSON/CSV/TSV via sparesults.

Decision: closed `ResultsFormat` enum for Xml, Json, Csv, Tsv with name and
media-type lookup. Unknown aliases return `Unsupported`. Graph query results
use RDF `Serializer` from 0.2, not SPARQL results formats.

Evidence: `src/query.rs` (`ResultsFormat`), `tests/query.rs`.

Revisit when: additional W3C result formats must be advertised.

### ADR-012 — Query cancellation policy

State: accepted  
Milestone: 0.3

Context: Architecture requires a documented cancellation policy by 0.3.
Oxigraph provides `CancellationToken`.

Decision: `Query`/`Update` accept an optional `CancellationToken`. Cancelling
the token requests cooperative abort during evaluation. Wall-clock timeouts are
**not** a facade feature—callers spawn a timer and cancel the token. Absence of
a token means no cooperative cancel.

Evidence: rustdoc on `Query::cancellation_token`, `tests/query.rs`,
`docs/users/sparql.md`.

Revisit when: a first-class timeout API is required for C consumers.

## Proposed decisions

*(none)*

## Accepted decisions (continued)

### ADR-022 — Sealed durable adapter and optional backend matrix

State: accepted

Date: 2026-07-31

Milestone: 0.8

Context: ADR-006 made Fjall the only supported durable engine and explicitly
set this revisit trigger. Applications may need a different embedded
key-value engine for pure-Rust policy, existing native dependencies,
operational tooling, file layout, or workload characteristics. Adding engine
branches directly to `Model` would duplicate transaction/error behavior and
freeze backend details into the future C ABI.

Decision:

- Extract Fjall behind a private sealed durable-store adapter and make its
  existing format-v1 suite the initial conformance baseline.
- Keep `Model::open(path)` selecting Fjall for compatibility; add explicit
  typed backend selection through `OpenOptions`.
- Plan first-party optional adapters for redb, RocksDB, SQLite, and LMDB. Keep
  native engines out of default features.
- Run bounded promotion evaluations for sled, LevelDB, MDBX, and SurrealKV
  because they fit the embedded byte-key model but have stability, binding,
  capability, or packaging questions.
- Require atomic batch commit, durable sync, full scan, layout validation,
  crash recovery, and common error/capability semantics from every promoted
  backend.
- Preserve standards RDF as the cross-backend archive. Native directories are
  neither auto-detected nor treated as mutually portable.
- Decide separately before 0.10 whether the proven sealed adapter can become a
  safe public user-supplied trait. Dynamic native plug-in loading remains out
  of scope.

Alternatives considered:

- Keep Fjall as the only durable engine.
- Add one-off engine-specific `Model` implementations and constructors.
- Publish a custom backend trait immediately, before multiple implementations
  have tested its semantic and object-safety boundary.
- Treat remote/server databases as if they obeyed the local synchronous path
  contract.

Consequences:

- Storage selection becomes a 0.8–0.10 cross-cutting track and must be designed
  before C ABI backend identifiers stabilize.
- The CI and package matrix grows only for explicitly enabled adapters.
- A popular engine can still be deferred or rejected if it cannot satisfy the
  common durability contract; canonical names never silently fall back.
- Existing Fjall users keep their constructor and format-v1 reader.

Evidence:
[storage backend expansion plan](design/storage-backend-expansion.md), sealed
adapter in `src/storage/`, shared backend conformance tests, and per-evaluation
promotion records.

Revisit outcome: ADR-024 freezes the first-party matrix and keeps the adapter
sealed for 1.0.

### ADR-023 — C ABI ownership, panic, and allocator contract

State: accepted  
Date: 2026-07-31  
Milestone: 0.8

Context: Redland C programs rely on opaque handles, paired alloc/free, and
non-unwinding FFI. The safe `oxiland` crate forbids `unsafe` (ADR-002). The
0.8 preview must be auditable without claiming full ABI drop-in (0.9).

Decision:

- Implement all C exports in `crates/oxiland-capi` with opaque tagged handles.
- Every `extern "C"` entry contains panics via `catch_unwind` and never
  unwinds into C.
- Returned C strings/buffers use one documented allocator and are freed only
  with `librdf_free_memory`; handles use typed `librdf_free_*`.
- Null checks, type tags, invalid UTF-8 rejection, and double-free defenses are
  required for every exported pointer type.
- Publish a per-handle thread-safety matrix ([0.8-cabi.md](design/0.8-cabi.md)).
- Freeze a preview symbol allowlist in the milestone plan; unsupported Redland
  APIs are omitted from headers rather than stubbed as silent no-ops.

Consequences:

- Sanitizer and export-allowlist CI are 0.8 release gates.
- Full symbol inventory closure and binary ABI claims remain 0.9.
- Python continues to bind the safe crate, not `oxiland-capi` (ADR-017).

Revisit when: expanding the allowlist in 0.9, or if a platform requires a
narrowly scoped exception to ADR-002.

### ADR-024 — Freeze first-party storage and keep the adapter sealed

State: accepted

Date: 2026-07-31

Milestone: 0.10

Context: the 0.10 storage gate must freeze backend identities, feature names,
capabilities, and layout-reader commitments. ADR-022 also required an explicit
decision on a public user-supplied `DurableBackend` trait after the first-party
adapters exercised the sealed boundary.

Decision:

- Freeze `memory`, `fjall`, `redb`, `rocksdb`, `sqlite`, and `lmdb` as the 1.0
  supported identities. Their Cargo features remain `storage-{name}` (with
  `storage-rocksdb` for `rocksdb`); Fjall remains the default durable backend.
- Expose feature-independent descriptors through `supported_backends()` and
  keep `compiled_backends()` for the adapters present in one build.
- Freeze `StorageCapabilities` and publish `LayoutReaderPolicy`: memory has no
  physical layout, while every durable adapter owns a format-v1 reader and
  standards-RDF export path.
- Reject a public custom-backend trait for 1.0. Keep `DurableStoreOps` sealed.
  The current boundary exposes engine initialization, layout mutation, and
  recovery operations whose panic, re-entry, crash-atomicity, and compensation
  invariants cannot be enforced by Rust's type system. Publishing it would
  also force `OpenOptions` to accept an open-ended identity while the C and
  Python registries promise a closed, auditable matrix.
- A future custom-backend API requires a separate provider object that owns its
  identity and layout version, a conformance kit with failure injection, and a
  SemVer boundary that does not expose first-party implementation hooks.

Alternatives considered: expose `DurableStoreOps` directly; expose an `unsafe`
trait; accept callbacks only in Rust while hiding them from C/Python; remove
all optional adapters and retain Fjall alone.

Consequences: applications choose among a stable first-party matrix or migrate
through N-Quads/TriG. A disabled first-party identity remains discoverable and
returns a specific unsupported error. Removing an adapter requires an export
window; it cannot strand the only readable copy of a supported layout.

Evidence: `src/storage/mod.rs`, `src/storage/durable.rs`,
`tests/backend_conformance.rs`, and
`docs/design/storage-backend-expansion.md` (SB-08).

Revisit when: after 1.0, only with a provider design and third-party prototype
that pass the full storage conformance/failure-injection suite.

### ADR-025 — Baseline factory registration for 0.10 parity

State: accepted  
Date: 2026-07-31  
Milestone: 0.10

Context: The 0.10 hard gate forbids in-scope safe exclusions. ADR-018 excluded
parser/serializer/storage/query factory registration from the safe facade, but
those `librdf_*_register_factory` symbols remain in the public Redland
denominator. Independent third-party plug-ins absent from the pinned baseline
profiles stay outside the denominator per `COMPATIBILITY.md`.

Decision:

- Supersede ADR-018 for 0.10+.
- Implement `register_*_factory` on the safe facade and C ABI for the closed
  set of built-in factories present in the pinned Redland baseline profiles
  (built-in syntaxes, SPARQL, and first-party storage backends).
- Re-registering a built-in name is idempotent and succeeds.
- Names outside the baseline built-in set fail observably (no silent success).
- Do not load arbitrary native plug-in modules or execute caller-supplied
  factory callbacks that would bypass Oxigraph/Oxiland safety boundaries.

Alternatives: keep ADR-018 exclusions (blocks 0.10); accept arbitrary `dlopen`
plugins; capability-error substitutes (forbidden by the hard gate).

Consequences: inventory factory rows become `verified` on safe and C surfaces.
Discovery enums remain the preferred Rust API; registration exists for Redland
workflow parity.

Evidence: `src/factory.rs`, C ABI factory exports, inventory 0.10 rows.

Revisit when: a supported third-party extension mechanism is required after 1.0.

### ADR-026 — Raptor and Rasqal world bridges for `librdf`

State: accepted  
Date: 2026-07-31  
Milestone: 0.10

Context: Redland exposes `librdf_world_get/set_raptor`, Rasqal equivalents, and
init-handler hooks. Independent Raptor/Rasqal APIs are outside the Oxiland
denominator, but bridges reached through `librdf` are in-scope for 0.10.

Decision:

- Store opaque bridge tokens on `World` (usize slots for raptor, rasqal, and
  their init handlers). Safe Rust never dereferences them.
- C ABI get/set functions read and write those tokens as `void *`.
- Oxiland parsing and SPARQL continue to use Oxigraph; the bridges exist for
  embedding parity and do not require linking stock Raptor/Rasqal.

Alternatives: exclude the bridges (blocks 0.10); link stock Raptor/Rasqal and
route I/O through them (rejected for the Oxigraph engine boundary).

Consequences: world bridge symbols become `verified`; embedding callers can
round-trip opaque handles.

Evidence: `src/world.rs`, `crates/oxiland-capi` world exports, inventory 0.10.

Revisit when: an embedding integration requires live Raptor/Rasqal callbacks.

### ADR-027 — Keep heed/bincode for optional LMDB in 0.10

State: accepted  
Date: 2026-07-31  
Milestone: 0.10

Context: cargo-audit reports that optional LMDB adapter dependency `heed`
still pulls unmaintained `bincode` 1.3.3 (R-023). That is a maintenance
warning, not a current RustSec vulnerability advisory. Replacing `heed` or
vendoring a different LMDB binding mid-qualification would churn the optional
backend matrix without closing a confirmed vulnerability.

Decision:

- Accept keeping `heed` (and inherited `bincode` 1.3.3) for the optional
  `storage-lmdb` feature through the 0.10 release candidate.
- Keep LMDB optional (never a default-feature dependency).
- Continue tracking upstream `heed` releases and RustSec for `bincode`;
  upgrade or replace under the storage conformance suite when a maintained
  path lands or a vulnerability is published.
- Do not treat the maintenance warning alone as a 0.10 release blocker.

Alternatives: drop LMDB from 0.10; replace `heed` immediately; vendor a fork
that drops `bincode`.

Consequences: R-023 is mitigated (accepted residual maintenance exposure on
an optional feature). Default and Fjall-only builds are unaffected. A real
advisory or unsupported-toolchain break reopens contingency under R-023.

Evidence: `docs/RISKS.md` R-023, optional `storage-lmdb` feature, cargo-audit
CI lockfile coverage.

Revisit when: `heed` drops or replaces `bincode`, a RustSec advisory lands, or
post-1.0 LMDB packaging review.

### ADR-028 — 0.12 competitive-parity performance gate

State: accepted  
Date: 2026-08-03  
Milestone: 0.12

Context: The 0.10/0.11 scaffold froze a “faster-than-Redland” rule requiring
Oxiland/Redland median throughput ≥ `1.05` (latency ≤ `0.95`) with the 95%
bootstrap CI excluding parity. Native tip builds under the matched
production-compile protocol (`cargo build --release` with thin LTO, C wrappers
at `-O3 -march=native`, validated `perf_bench` workloads, and C hot-path
optimizations) measure **near parity** against system/`librdf` (typically
0.97–1.03). Multi-× wins in older 0.11 samples are not reproducible under that
protocol. Keeping an unreachable 5% win margin would force fabricated ratios or
unequal builds (R-022).

Decision:

- For milestone 0.12, freeze a **competitive-parity** gate:
  - throughput: median Oxiland/Redland ≥ `0.90`, and 95% bootstrap CI lower
    bound `> 0.85`;
  - latency: median Oxiland/Redland ≤ `1.20`, and 95% bootstrap CI upper bound
    `< 1.40`.
  - Samples: at least 40 independent timed iterations per case.- Retain production-compile provenance, independent samples, no case deletion,
  and RSS budgets.
- Do not market a blanket “faster than Redland” claim from 0.12 alone; publish
  per-case ratios.
- A later ADR may restore a stricter faster-than-Redland margin when matched
  evidence sustains it.

Alternatives considered: keep 1.05 and block 0.12 forever; delete cases;
compare against deliberately slower Redland builds.

Consequences: `0.12-suite.json` and `check-performance-gate.py` use these
thresholds when evaluating the 0.12 suite; verification/charter cite this ADR.

Evidence: tip `perf_bench` release measurements after cardinality cache, handle
hot-path, stream amortization, and memory insert coalescing
(`docs/reports/0.12.md`).

Revisit when: Oxigraph/Oxiland sustain ≥1.05 on every required case under the
same matched protocol.

### ADR-029 — 0.13 suite-wide faster-than-Redland gate

State: accepted  
Date: 2026-08-04  
Milestone: 0.13 / 1.0 readiness

Context: ADR-028 froze a competitive-parity gate for 0.12 and deferred a
blanket faster-than-Redland claim until matched evidence sustained the
stricter margin (throughput ≥ `1.05`, latency ≤ `0.95`, bootstrap CI
excluding parity). The corrected paired driver
(`perf_bench_0_13.c` / `scripts/run-0.13-performance.py`) and tip host-scoped
wins make that claim testable. Suite-wide authorization still requires three
independent corrected-runner passes on every required host.

Decision:

- Freeze `compatibility/performance/0.13-suite.json` with the historical
  faster-than-Redland thresholds (throughput median ≥ `1.05` and CI lower
  `> 1.0`; latency median ≤ `0.95` and CI upper `< 1.0`), 100 paired samples,
  and RSS budgets at `1.25`.
- Require three independent runs per target (Linux x86-64, macOS Apple Silicon,
  Windows x86-64), collected via `.github/workflows/qualify-0.13.yml` and
  checked by `scripts/check-0.13-release.py`.
- Do not weaken thresholds to turn a loss green; red CI means the suite-wide
  claim stays unauthorized.
- ADR-028 remains the closed 0.12 competitive-parity record; 0.13 does not
  reopen or rewrite that gate.

Alternatives considered: keep competitive-parity forever; accept a single-host
or single-run win as suite-wide; auto-commit evidence without a fail-closed
checker.

Consequences: marketing and evaluator docs may claim suite-wide
faster-than-Redland after nine cells pass under this ADR. Local
host-scoped tables remain separately scoped.

Evidence: committed nine-cell bundle under
`compatibility/qualification/performance/0.13/`,
[qualify-0.13 run 30973969324](https://github.com/eddiethedean/oxiland/actions/runs/30973969324)
on `a50ee5b25eb9daa56b0cf1d155856e1c312b35fb`,
`compatibility/qualification/0.13-matrix.json`, and
`scripts/check-0.13-release.py` green.

Revisit when: a required host cannot sustain the margin under the matched
protocol, or 1.0 readiness defers the claim with documented scope.

### ADR-017 — Python package is Pythonic, not a thin Rust mirror

State: accepted  
Date: 2026-07-31  
Milestone: 0.7

Context: Roadmap 0.7 ships a PyPI package over the frozen 0.6 safe Rust
facade. Callers need idiomatic Python, not a mechanical mirror of Rust builders
or a Redland/`rdf` CPython binding clone.

Decision:

- Publish package name `oxiland` from monorepo `python/` via maturin + PyO3
  `cdylib`, path-depending on the safe `oxiland` crate (not `oxiland-capi`).
- The extension crate is **not** a Cargo workspace member of the root (root
  keeps `#![forbid(unsafe_code)]`; all FFI `unsafe` stays in the PyO3 crate).
- Prefer kwargs / module functions over fluent Rust builders; expose
  `with model.transaction()` as a context manager; use the Python iterator
  protocol for find, parse, SELECT, and CONSTRUCT streams without forced
  materialization.
- Map `Error` variants (`src/error.rs`) to a typed exception hierarchy under
  `OxilandError` (not stringly-only failures).
- Accept `pathlib.Path` / path-like and `str`/`bytes` for file and buffer I/O.
- Support CPython **3.10–3.14**; wheel builds via **maturin-action** on
  ubuntu/macOS/windows hosts (CPython 3.10–3.14). Residual platforms
  (dedicated aarch64 manylinux runners, etc.) remain optional expansions;
  0.7.0 publishes the CI-verified wheels through PyPI Trusted Publishing and
  does not publish an sdist because the path dependency on the Rust crate
  cannot ship a usable source archive from `python/` alone.
- Treat wheels as first-class release artifacts: validate their Python
  metadata, license files, native ABI tags, PEP 561 surface, and embedded
  CycloneDX SBOM; install-smoke every platform/interpreter pair; attest the
  exact tested files; and publish them with a SHA-256 manifest on the GitHub
  release. The release workflow never rebuilds wheels after CI.
- **Defer rdflib interop** for 0.7 (no convert helpers, no store adapter, no
  behavioral-identity claim). Revisit in a later ADR if needed.
- Do **not** claim Redland Python binding drop-in compatibility or CPython ABI
  stability tied to `oxiland-capi`.
- Query cancellation tokens are omitted from the 0.7 Python surface (callers
  may interrupt at process level); document as a non-mirror.

Consequences:

- Dual maintenance of Rust facade + Pythonic surface; design note
  [`0.7-python-api.md`](design/0.7-python-api.md) lists intentional non-mirrors.
- Python versioning tracks the 0.7.x train alongside the Rust crate where
  practical.
- Typing (`py.typed` / stubs), pytest, artifact integrity, and wheel provenance
  are release gates.

Revisit when: adding rdflib interop, changing the supported CPython matrix, or
layering Python on a future C ABI.

### ADR-006 — Persistent storage compatibility boundary

State: accepted  
Date: 2026-07-30  
Milestone: 0.4

Decision: Oxiland promises a **versioned Oxiland on-disk format** for
Fjall-backed `Model::open`, not raw Oxigraph store-directory compatibility and
not silent forever-forward binary compatibility across Oxiland major versions.

Format v1 stores an `__oxiland/meta` JSON document (`format_version: 1`) beside
N-Quads quad keys in the Fjall `oxiland_quads` partition. Patch releases in the
**0.4.x–0.12.x** lines must open format v1 without migration. Pre-0.4 experimental
stores (no metadata) are opened only via `Model::migrate_legacy_store`, which
rewrites metadata after validating parseable quad keys; otherwise callers receive
`Unsupported` with N-Quads archival guidance.

Archival continuity is standards RDF (N-Quads/TriG), not Fjall directories. Export
N-Quads before any future format-v2 migration.

Alternatives considered:

- Logical-only compatibility with no on-disk promise (rejected: blocks the 0.4
  reopen/migrate evidence gate and user upgrade stories).
- Pin Oxigraph RocksDB directories as the durable API (rejected: Oxiland uses Fjall
  quad keys + Oxigraph memory working set; would couple the wrong artifact).
- Silent auto-migration on every `open` (rejected: surprising durable rewrites;
  prefer an explicit migrate entry point).

Consequences:

- `Model::open` requires format v1 or initializes it for empty new stores.
- User docs stop calling Fjall “experimental.”
- R-016 mitigated for 0.4.x–0.12.x; major bumps may introduce format v2 with a
  documented migrator. The reopen window was extended through 0.12 without a
  format bump; `SUPPORT.md` is the canonical user-facing statement.

Evidence: `docs/design/0.4-storage-api.md`, `src/storage/fjall.rs`,
`tests/storage.rs`.

Revisit when: introducing format v2 or a second durable backend.

### ADR-013 — Shared streaming policy without a unifying trait

State: accepted  
Date: 2026-07-30  
Milestone: 0.5

Context: ADR-005 and ADR-010 left open whether find/parse/query streams should
share a trait. Three mature iterator shapes already exist.

Decision: document a shared fallible-iterator policy (lazy
`Iterator<Item = Result<_>>`, early-stop by drop) without introducing a unifying
trait. Lending iterators and Redland callback visitors remain deferred.

Alternatives: unifying `FallibleStream` trait; callback visitors.

Consequences: no API churn for existing streams; 0.5 inventory verifies the
policy via early-stop tests. C ABI stream handles remain 0.8.

Evidence: `docs/design/0.5-streams-utilities.md`, `docs/users/streams.md`,
`tests/model.rs`, `tests/io.rs`, `tests/query.rs`.

Revisit when: C ABI or inventory forces a shared trait.

### ADR-014 — World logging facade and optional tracing

State: accepted  
Date: 2026-07-30  
Milestone: 0.5

Context: Redland exposes log levels/facilities and callbacks. Oxiland needs a
safe Rust equivalent without a global mutable logger.

Decision: attach logging to `World` (`LogLevel`, `LogFacility`,
`set_log_handler`, `log`). Clones share the feature registry, minimum log
level, and handler (`Arc`). Optional Cargo feature `tracing` also emits
`tracing` events, gated by the same minimum level as the handler. Callback
ordering is synchronous and deterministic for a single composed handler.

Alternatives: `log` crate only; process-global logger; no callbacks.

Consequences: tests can assert ordering; apps opt into `tracing` when desired.

Evidence: `src/world.rs`, `tests/utility.rs`.

Revisit when: async/structured logging requirements exceed sync callbacks.

### ADR-015 — Closed digest algorithm set

State: accepted  
Date: 2026-07-30  
Milestone: 0.5

Context: Redland digests include MD5/SHA family helpers used in workflows, not
only security contexts.

Decision: support `md5`, `sha1`, and `sha256` via `utility::DigestAlgorithm`.
Unknown names return `Error::Unsupported`. Digests are always available in the
default build.

Alternatives: feature-gated crypto; OpenSSL bindings; open-ended algorithm
registry.

Consequences: small always-on deps (`md-5`, `sha1`, `sha2`); security-sensitive
callers should prefer SHA-256.

Evidence: `src/utility/digest.rs`, `tests/utility.rs`.

Revisit when: inventory requires additional algorithms.

### ADR-016 — Hashes and lists map to standard Rust collections

State: accepted  
Date: 2026-07-30  
Milestone: 0.5

Context: Redland ships custom hash and list types tied to manual memory.

Decision: inventory curated hash/list/manual-memory symbols as
`not-applicable`. Callers use `HashMap`, `Vec`, and Rust iterators. Migration
examples document the mapping; Oxiland does not ship collection wrappers.

Alternatives: thin wrapper types; retain Redland-shaped mutable lists.

Consequences: simpler API; 0.6 accounting still lists remaining symbols.

Evidence: `docs/evaluators/migration-from-redland.md`,
`examples/std_replacements.rs`, inventory 0.5 `not-applicable` rows.

Revisit when: C ABI needs explicit list/hash handles.

Revisit outcome (0.10): the C ABI implements Redland-shaped `librdf_hash` /
`librdf_list` opaque handles over internal maps and vectors. Safe Rust rows
remain `not-applicable` with `safe_n_a_kind: "ownership-mechanic"`. Their C
forms must be `verified`.

### ADR-018 — Factory registration disposition for safe Rust

State: superseded  
Date: 2026-07-30  
Milestone: 0.6  
Superseded by: [ADR-025](#adr-025-baseline-factory-registration-for-010-parity)

Context: Redland exposes parser/serializer/storage/query factory registration
APIs. Architecture asked which registrations are safe and useful in Rust.

Decision: custom factory **registration** APIs (`librdf_*_register_*`,
plugin modules, Raptor world wiring for embedding) are **excluded** from the
safe facade. Callers use closed `Syntax`, `ResultsFormat`, and
`StorageBackend` discovery. Unsupported names return `Error::Unsupported`.
Built-in advertised formats remain first-class.

Alternatives: dynamic plugin loading; thin registration callbacks.

Consequences: simpler soundness story; inventory marks factory registration
`excluded` with migration to closed enums.

Evidence: `docs/design/0.6-safe-api-accounting.md`, `src/io/format.rs`,
`src/storage/mod.rs`.

Revisit when: a supported extension mechanism is required for 1.0.

Revisit outcome: superseded by ADR-025; its observable factory behavior is
reverified by the 0.11 full-parity gate.

### ADR-019 — `oxiland-cli` rdfproc workflow surface

State: accepted  
Date: 2026-07-30  
Milestone: 0.6

Context: ROADMAP requires rdfproc-equivalent command workflows.

Decision: ship workspace binary `crates/oxiland-cli` with rdfproc-shaped
commands (`parse`, `serialize`, `add`, `remove`, `find`, `query`, `contexts`,
`print`). Storage types are `memory` and `fjall` only. Not a binary/ABI
drop-in for native `rdfproc`.

Alternatives: feature-gated bin in the library crate; docs-only recipes.

Consequences: ARCHITECTURE workspace layout begins; CI runs CLI smoke.

Evidence: `docs/design/0.6-cli-rdfproc.md`, `crates/oxiland-cli`.

Revisit when: packaging a homebrew/apt `rdfproc` replacement name.

### ADR-020 — 1.0 naming and module freeze intent

State: accepted  
Date: 2026-07-30  
Milestone: 0.6

Context: 0.6 freezes naming conventions intended for 1.0 before Python (0.7)
and C ABI (0.8) bind the facade.

Decision: public modules `terms`, `io`, `storage`, `utility` (incl. `vocab`),
root re-exports (`Model`, `World`, `Query`, `Update`, `Error`, …), and the
closed `Error` variant set are frozen for 1.0 intent. Breaks require ADR +
CHANGELOG. Advanced Oxigraph escapes remain under `io::primitives` and
`sparql` (re-exported primitives module).

Alternatives: continue renaming freely until 0.10.

Consequences: semver-checks against 0.5.0+ become meaningful.

Evidence: `docs/design/0.6-safe-api-accounting.md`, `api/oxiland-public-api.txt`.

Revisit when: 0.10 RC scope review.

### ADR-021 — Header-derived inventory generation

State: accepted  
Date: 2026-07-30  
Milestone: 0.6

Context: Curated milestone slices cannot claim full safe-API accounting.

Decision: generate public `librdf_*` function symbols from pinned Redland
1.0.17 headers (`scripts/generate-redland-inventory.py`). Inputs are
checksummed. Checked-in classifications are authoritative; regen merges by ID
and must not wipe human classifications without review. Milestone 0.6 forbids
`unreviewed` and residual `mapped` states at exit.

Alternatives: continue curated slices only; vendor full Redland trees in-repo.

Consequences: inventory size grows to hundreds of rows; shared accounting
tests evidence N/A and excluded families.

Evidence: `compatibility/baseline/redland-1.0.17.sha256`,
`compatibility/inventory/redland-1.0.17-oxiland-0.6.json`.

Revisit when: rebasing to a newer Redland reference API.

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
