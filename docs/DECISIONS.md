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
`oxiland-capi` no earlier than 0.8 as the only legacy C ABI boundary. The
planned Python package (roadmap 0.7) binds the safe Rust crate directly and is
not layered on `oxiland-capi`.

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

Revisit when: zero-copy lending access is required, or query streams in
0.3–0.5 need a shared streaming trait.

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

Revisit when: a shared streaming trait across find/parse/query is required
(0.5).

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

### ADR-013 — Python package is Pythonic, not a thin Rust mirror

State: proposed  
Decision deadline: before 0.7 public beta

Question: how closely should the PyPI package mirror Rust builders versus
idiomatic Python (kwargs, context managers, exception types, iterators)?

Evaluation criteria:

- ergonomics for Python RDF/SPARQL applications;
- maintenance cost of dual surfaces;
- typing and documentation quality;
- whether rdflib or other ecosystem interop is in scope for 0.7;
- clear non-goals (no C ABI layering; no claim of Redland Python binding
  drop-in unless separately evidenced).

## Accepted decisions (continued)

### ADR-006 — Persistent storage compatibility boundary

State: accepted  
Date: 2026-07-30  
Milestone: 0.4

Decision: Oxiland promises a **versioned Oxiland on-disk format** for
Fjall-backed `Model::open`, not raw Oxigraph store-directory compatibility and
not silent forever-forward binary compatibility across Oxiland major versions.

Format v1 stores an `__oxiland/meta` JSON document (`format_version: 1`) beside
N-Quads quad keys in the Fjall `oxiland_quads` partition. Patch releases in the
0.4.x line must open format v1 without migration. Pre-0.4 experimental stores
(no metadata) are opened only via `Model::migrate_legacy_store`, which rewrites
metadata after validating parseable quad keys; otherwise callers receive
`Unsupported` with N-Quads archival guidance.

Archival continuity is standards RDF (N-Quads/TriG), not Fjall directories.

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
- R-016 mitigated for 0.4.x; major bumps may introduce format v2 with a
  documented migrator.

Evidence: `docs/design/0.4-storage-api.md`, `src/persist.rs`,
`tests/storage.rs`.

Revisit when: introducing format v2 or a second durable backend.

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
