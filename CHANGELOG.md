# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.12.0] - 2026-08-03

Competitive-parity performance gate (ADR-028) on the three-host matrix:
production-compile evidence, frozen RSS budgets, C hot-path optimizations, and
retained 0.11 parity. Does not authorize a blanket faster-than-Redland claim.

### Added

- Frozen `compatibility/performance/0.12-suite.json` and
  `compatibility/qualification/0.12-matrix.json` with RSS budgets and ADR-028
  thresholds
- `scripts/build-0.12-performance-evidence.py` and `.github/workflows/qualify-0.12.yml`
- Draft strict `compatibility/performance/0.13-suite.json`, calibrated C driver,
  and genuinely paired per-sample AB/BA evidence runner covering ten public C
  API workloads with 100 samples per implementation
- Paired bootstrap confidence intervals and isolated five-sample median
  peak-RSS probes in the performance qualification tooling
- ADR-028 competitive-parity performance decision
- `docs/reports/0.12.0-release.md`
- C lifecycle regression coverage for progressive partial parses, configured
  query limit/offset, lazy SELECT values, deferred stream materialization, and
  stale model-size/double-free rejection

### Changed

- C hot paths: cardinality cache, hot handle borrow, fast `librdf_model_size`,
  deferred stream statement materialization, query CONSTRUCT stream handoff
- In-memory C parsing uses direct string parsing and bounded 4,096-quad bulk
  transactions while preserving Redland's progressive partial-load behavior
- Query handles retain their configured safe query instead of cloning query
  text on execution; SELECT rows use one flat allocation and C binding nodes
  materialize only when a binding getter observes them
- Full-model streams use known cardinality for constant-time unobserved
  traversal and defer store-row decoding until `librdf_stream_get_object`
- C model serialization writes directly into the growable `malloc` buffer
  returned to callers instead of copying through a `Vec`, `String`, and
  duplicated C string
- Model-size calls use world-owned tagged tombstones for safe stale-pointer
  rejection and a direct cardinality fast path; handle registration no longer
  invalidates unrelated hot handles, with a generation-bound process cache on
  Windows
- Memory model inserts coalesce into a single Oxigraph transaction on flush
- Pending in-memory insertion buffers reserve an initial 1,024 entries to avoid
  repeated growth on common bulk workloads
- Release profile uses fat LTO and a single codegen unit; C perf harness builds
  at `-O3 -march=native`
- `scripts/check-performance-gate.py` reads suite thresholds (legacy 1.05/0.95
  suites keep CI-above-parity; 0.12 uses competitive-parity bounds) and
  resamples matched observations for paired protocols
- Performance evidence isolates Oxiland and Redland runtime library paths,
  records independently calibrated workload samples, and measures each RSS
  probe as the sole child of a fresh process
- Milestone 0.12 docs, user performance guide, and R-022 closed for this gate

### Fixed

- Performance harness validates workloads; `P-GRAPH-10K` runs a real CONSTRUCT
- `P-SER-NQ-10K` now exercises actual N-Quads serialization
- C query limit and offset setters now affect the executed query
- Redland-linked benchmark processes can no longer preload Oxiland's
  compatibility library through `DYLD_LIBRARY_PATH` or `LD_LIBRARY_PATH`
- Unix RSS evidence no longer reuses the cumulative high-water mark from an
  earlier child process

## [0.11.0] - 2026-08-03

Demonstrated Redland parity: frozen baseline and obligations, six-cell native
two-sided differentials, fail-closed release checker, C source corpus,
librdf-compatible packaging, and revision-bound soak/performance evidence.

### Added

- Checksummed Redland 1.0.17 header/export baseline and 1,532 behavior obligations
- Two-sided harness (`compatibility/harness/two_sided.py`) emitting revision-bound
  raw observations under `compatibility/qualification/raw/`
- Fail-closed `scripts/check-0.11-release.py` and `scripts/derive-0.11-parity.py`
- Frozen C corpus and librdf-compat packaging (`scripts/package-librdf-compat.sh`)
  with ABI-swap smoke
- Obligation-linked safe Rust tests (`tests/obligations_0_11.rs`)
- Cross-platform qualification workflow (`.github/workflows/qualify-0.11.yml`)
- Six-cell raw + native performance qualification bundle (Linux, macOS, Windows)

### Changed

- Workspace, CLI, C, and Python package versions aligned at `0.11.0`
- C ABI soft stubs: tagged iostreams, factory callbacks, parser features, log locator
- `symbols.version` exports `OXILAND_0.11` with `LIBRDF_1.0.17` alias
- User docs retire “preview” framing where 0.11 gates pass; remaining behavioral
  gaps stay documented in `docs/users/c-abi-limitations.md`

## [0.10.0] - 2026-07-31

Frozen 1.0-intent storage contracts, expanded the Redland-shaped C preview,
and introduced fail-closed qualification-scaffold tooling.

### Added

- Frozen, feature-independent storage descriptors and explicit format-v1
  layout-reader commitments for the intended 1.0 backend matrix
- Strict machine-readable candidate-coverage and performance qualification
  validators, a frozen target/profile matrix, and regression tests for the
  no-waiver/no-averaging rules
- ADR-024 decision to keep the physical durable adapter sealed for 1.0
- ADR-025 baseline factory registration (supersedes ADR-018); ADR-026 Raptor
  and Rasqal world bridges; ADR-027 optional LMDB/heed decision (R-023)
- Full C ABI allowlist covering the public `librdf_*` denominator (`OXILAND_0.10`)
  with lifecycle coverage and Linux ASan/LSan (rustc no longer ships UBSan)
- Safe Rust storage facade, feature get/set, and factory registration APIs
- 0.10 qualification scaffold: inventory, six-profile candidate evidence,
  three synthetic performance fixtures, soak and fuzz-smoke records, and a
  Python stub snapshot

### Changed

- Workspace, CLI, C, and Python package versions are aligned at `0.10.0`
- The 0.10 roadmap state is `complete`; `scripts/check-0.10-release.py` passes
- Storage capability construction is available uniformly through
  `StorageCapabilities::for_backend`
- Risk register records mitigations backed by inventory, C ABI, storage, and
  crash evidence while leaving native performance proof active for 0.11

### Fixed

- Inventory validation compares numeric milestone components, so `0.10`
  correctly sorts after `0.9` and receives C ABI schema validation

## [0.9.0] - 2026-07-31

Expanded C source-compatibility, optional durable-store adapters, and
downstream validation.

### Added

- Expanded `oxiland-capi` allowlist, lifecycle helpers, and C downstream
  smoke coverage
- Optional `redb`, RocksDB, SQLite, and LMDB durable-store adapters behind
  explicit Cargo features, with backend conformance coverage
- Downstream C consumer harnesses and a 0.9 release checklist

### Changed

- Crate and Python package versions are aligned at `0.9.0`
- CLI and C API storage feature flags forward to the core crate
- Active release documentation now distinguishes the published 0.8 line from
  the unreleased 0.9 tip

### Fixed

- Core builds and tests correctly with default features disabled
- C handles are validated before dereference or free, and logging callbacks may
  re-enter the API without holding the logger lock
- Invalid C language tags fail explicitly; digest input validation rejects null
  buffers with nonzero lengths
- Multi-key durable-store replacement operations are atomic on supported
  backends and iterator errors are no longer silently discarded

## [0.8.0] - 2026-07-31

Auditable C ABI source-compat preview and sealed durable-store adapter (ADR-022/023).

### Added

- crates/oxiland-capi with preview librdf_* allowlist, headers, pkg-config, representative C example
- Sealed DurableStore adapter; format_v1 helpers; backend conformance harness
- OpenOptions::new(StorageBackend, path); compiled_backends(); known-but-not-compiled backend errors
- CI: C ABI job (example, symbol allowlist, ASan)
- Inventory redland-1.0.17-oxiland-0.8.json with c_abi/c_state fields
- docs: milestones/0.8.md, design/0.8-cabi.md, design/0.8-redb-spike.md, reports/0.8.md, users C guide

### Changed

- Fjall persistence moved behind src/storage/{fjall,durable,format_v1}.rs
- StorageBackend::from_name distinguishes known-uncompiled vs unknown vs legacy
- Crate versions 0.8.0 (Rust, CLI, CAPI, PyPI aligned)

## [0.7.0] - 2026-07-31

First-class typed Python package for embedded RDF models, persistence, SPARQL,
and streaming I/O (ADR-017).

### Added

- `python/` maturin + PyO3 package packaged for PyPI as `oxiland`
- Idiomatic Python APIs for terms, `Model`, parse/serialize, SPARQL
  query/update, transactions, and curated digests/vocab helpers
- Typed exception hierarchy aligned with Rust `Error` categories
- PEP 561 stubs (`oxiland.pyi` / `py.typed`); pytest + pyright CI
- Wheel builds on Linux/macOS/Windows for CPython 3.10–3.14
- Standalone Python documentation track and runnable `python/examples/`
- ADR-017 accepted; milestone/design/report docs for 0.7

### Changed

- Crate version `0.7.0` (Rust + `oxiland-cli` + PyPI aligned)
- Parity ledger / roadmap mark 0.7 complete; next focus 0.8 C ABI
- Project, Rust, Python, CLI, production, contributor, support, security,
  architecture, and verification documentation upgraded into task-oriented
  product contracts
- Crate, wheel, module, and CLI descriptions now lead with their user-facing
  RDF capabilities; Redland mapping remains an explicit migration surface

### Fixed

- Python transactions reject independently-created nested contexts so an inner
  context cannot commit through an outer rollback
- Wheel CI installs every advertised OS/interpreter artifact; PyPI publication
  reuses those verified wheels through Trusted Publishing after crates.io
- Python extension upgraded to PyO3 0.29.0 to remove the 0.25 iterator and
  closure soundness advisories
- Tip CI runs the same unwaived `cargo audit`, version alignment,
  `cargo publish --dry-run`, and full 15-wheel checks used before release

## [0.6.0] - 2026-07-30

Header-derived safe-API accounting, `oxiland-cli` rdfproc workflows, and 1.0
naming freeze intent over Oxigraph 0.5.9.

### Added

- Full public `librdf` 1.0.17 function inventory
  (`redland-1.0.17-oxiland-0.6.json`) generated from pinned headers (ADR-021)
- Workspace binary crate `oxiland-cli` for rdfproc-shaped parse/find/query/
  serialize workflows (ADR-019)
- Symbol map + expanded migration guide; CLI user docs
- ADR-018–ADR-021 (factories, CLI, naming freeze, inventory generation)
- `cargo semver-checks` CI gate against published 0.5.0
- Compatibility report `docs/reports/0.6.md` and API review checklist

### Changed

- Crate version `0.6.0`; Cargo workspace layout (`oxiland` + `oxiland-cli`)
- Parity ledger reports 100% **safe-API accounting** (not C ABI)
- `Error` / module layout documented as frozen for 1.0 intent (ADR-020)

### Fixed

- CLI defaults to `nquads` so named-graph print/find/serialize succeed
- CLI `-n` gates Fjall `create`; missing paths fail without it
- CLI `-s` accepts only `memory` / `fjall` (no silent `hashes`/`file` aliases)
- `parse-stream` uses progressive `load_path_into`; `parse` stays collecting
- Query language must be `-`/`sparql`; typed/lang literal CLI args are rejected
- Inventory no longer marks missing feature/storage APIs as `verified`
- Generator `--check-only` diffs against checked-in 0.6 classifications

## [0.5.0] - 2026-07-30

Streams policy documentation, utilities, digests, vocabulary helpers, and World
logging over Oxigraph 0.5.9.

### Added

- Documented fallible-iterator stream policy (ADR-013); user guide
  `docs/users/streams.md`
- `oxiland::utility`: URI join/file helpers, Unicode NFC/NFKC, digests
  (MD5/SHA-1/SHA-256, ADR-015), `Namespace`, curated `vocab` constants
- `World` logging: `LogLevel`, `LogFacility`, handlers; optional `tracing`
  feature (ADR-014)
- Hash/list → std migration example `std_replacements` (ADR-016)
- Inventory `redland-1.0.17-oxiland-0.5.json`, design doc, ADRs 013–016
- Compatibility report `docs/reports/0.5.md`; utility digest smoke harness

### Changed

- crates.io description covers utilities
- Parity ledger marks digests/logging/heuristics/hashes for 0.5

### Fixed

- `join_iri` / `relativize_iri` no longer treat `://` as the path root
  (authority-only bases such as `https://example.com` resolve correctly)
- `file_uri_to_path` strips query/fragment before decoding; clearer UTF-8
  path errors; Windows drive `:` and UNC round-trip
- `Namespace::new` requires bases ending in `/`, `#`, or `:`
- `World` clones share minimum log level; `tracing` emission uses the same gate
- Docs/ADR numbering: Python ADR renumbered to ADR-017; ADR-005/010 revisit
  notes align with ADR-013; format v1 promise covers 0.5.x
- Vocab helpers live under `utility::vocab::{rdf,rdfs,xsd,owl,dc}` modules
- BOM probe preserves partially read bytes across `Interrupted` / other I/O errors
- `create(false)` / read-only open recognize only Fjall layout markers (no
  pollution of unrelated nonempty directories)
- Disk replace/insert/clear compensation propagates undo failures
- `import_nquads_from_path` maps mid-stream I/O to `Error::Io` (not `Parse`)

## [0.4.0] - 2026-07-30

Durable Fjall storage contract, transactions, and archival helpers over Oxigraph
0.5.9.

### Added

- Oxiland on-disk **format v1** metadata (`__oxiland/meta`) and
  `Model::migrate_legacy_store` for pre-0.4 experimental directories (ADR-006)
- `OpenOptions`, `StorageBackend`, `StorageCapabilities`
- `Model::open_with`, `transaction` / `ModelTransaction`, `sync`, `clear`,
  `clear_graph`, `bulk_insert_quads`
- Read-only Fjall opens; `Parser::load_transactional` /
  `load_path_transactional`
- N-Quads archival `export_nquads_to_path` / `import_nquads_from_path`
- Inventory `redland-1.0.17-oxiland-0.4.json`, design docs, example
  `persistent_transaction`
- Compatibility report `docs/reports/0.4.md`

### Changed

- Fjall persistence is a supported 0.4 contract (no longer experimental)
- Model locking uses `RwLock` so readers do not observe mid-reload empty stores
- crates.io description covers durable storage
- User persistence guide rewritten for format v1 and transactions

### Fixed

- Panic inside `Model::transaction` clears the in-transaction flag via `Drop`
  so the model stays writable afterward
- Same-thread `len` / `find` / `Query::execute` during a transaction no longer
  deadlock on the non-reentrant `RwLock` (reads see the last committed set)
- `OpenOptions::create(false)` rejects empty / non-store directories instead of
  initializing Fjall there
- Read-only open refuses to initialize format metadata on empty paths
- UTF-8 BOM stripped for RDF parse streams, transactional load, and N-Quads
  import (`io::BomStrippingReader`)
- `clear_quads` compensates on SyncAll failure like insert/replace
- `migrate_legacy_store` drops the migrate handle before reopening
- Docs: transactional import is available; import merges (union); migration /
  getting-started / positioning / ADR-007 aligned with 0.4
- Public API snapshot tracks `BomStrippingReader`

## [0.3.0] - 2026-07-30

Redland-shaped SPARQL query, update, and results over Oxigraph 0.5.9.

### Added

- `Query` builder: base IRI, prefixes, limit/offset (algebra `Slice`), dataset
  selection, and cancellation token (ADR-009, ADR-012)
- Streaming ASK / SELECT / CONSTRUCT / DESCRIBE via `QueryResults` (ADR-010)
- `Update` facade with write-locked Fjall durable resync (rollback on sync
  failure)
- `ResultsFormat` (XML/JSON/CSV/TSV) plus ASK/SELECT serialize helpers and
  `serialize_graph_results_to_writer` (ADR-011)
- Owned `QueryResults` wrapper with non-draining `Debug` (ADR-010)
- Inventory `redland-1.0.17-oxiland-0.3.json`, design doc, ADRs 009–012
- Examples `construct` and `update`; SPARQL facade smoke harness
- Compatibility report `docs/reports/0.3.md`

### Changed

- crates.io description covers query/update/results
- User SPARQL guide documents 0.3 configuration and serialization
- Roadmap adds **0.7 Pythonic Python package** (not a 1:1 Rust port); C ABI
  preview moves to 0.8, downstream C to 0.9, RC to 0.10

### Fixed

- Update executes under the model write lock; Fjall resync compensates on
  mid-sync failure and rolls memory back to the pre-update disk snapshot
- Update dataset configuration returns `Unsupported` when USING datasets are
  unavailable (e.g. `INSERT DATA`)
- API `limit`/`offset` replace in-query `Slice` layers instead of nesting
- ASK rejects API `limit`/`offset` at builder time (including after PREFIX/BASE
  and a leading UTF-8 BOM)
- Invalid query/update base IRI and prefix map to `InvalidRdf` consistently
- SPARQL smoke harness exercises `compatibility/fixtures/sparql/smoke.ttl`
- Fjall durable keys use Oxigraph's canonical quad form and RDF-equal remove
  scanning so typed-literal lexical variants cannot resurrect deleted triples
- Duplicate RDF-equal inserts no longer write alternate lexical keys to disk
- Durable insert/remove SyncAll failures compensate partition mutations; model
  reloads from disk on persist errors instead of assuming the write never stuck
- SPARQL Update replace compensation propagates failures instead of swallowing them
- `serialize_model_to_path` flushes the `BufWriter` and surfaces flush errors
- `serialize_graph_results_to_writer` streams triples without buffering the graph
- `World` recovers from poisoned feature locks like `Model`
- Progressive-load annotations cover all `Error` variants
- Security support matrix, 0.3 evidence gates, and harness/inventory docs aligned
  with the shipped 0.3 facade smoke (not native Rasqal differentials)

## [0.2.0] - 2026-07-30

Redland-shaped RDF input and output over Oxigraph 0.5.9.

### Added

- Closed `Syntax` discovery by name, media type, and extension (ADR-008)
- Streaming `Parser` with base IRI, `GraphTarget`, reader/slice/string/path
  entry points, and blank-node renaming
- Progressive `load_into` and collecting `load_collecting` model loads (ADR-007)
- Streaming `Serializer` with namespace prefixes, writer/string/path helpers,
  and graph-versus-dataset checks
- Structured `Error::Parse` / `Serialize` / `Io` categories and `ParseError`
- Curated 0.2 I/O inventory, format matrix, conformance fixtures, and
  `rapper` oracle/differential harnesses
- Example `parse_serialize`
- Compatibility report `docs/reports/0.2.md`

### Changed

- `oxiland::io` is now the Redland-shaped facade; Oxigraph primitives moved to
  `oxiland::io::primitives`
- Public API snapshot expanded for the I/O surface

### Fixed

- `GraphTarget::DefaultGraph` rejects named-graph input so it differs from
  `Dataset` on TriG/N-Quads; `GraphTarget::Named` remaps the default graph and
  rejects foreign named graphs
- Parse errors no longer duplicate embedded Oxigraph location text
- `load_collecting` rolls back quads newly inserted by the call if a later
  insert fails
- Model serialization streams statements instead of buffering a full copy
- Ambiguous aliases (`text/plain`, `application/xml`, `.txt`, `.xml`) return
  `Unsupported` instead of guessing
- Namespace prefixes are rejected on formats that cannot emit them

### Compatibility claims

- Provides the 0.2 RDF I/O surface described in `docs/reports/0.2.md`
- Does **not** claim full Redland API accounting, C source/ABI compatibility,
  N3/JSON-LD support, or transactional atomic import

## [0.1.0] - 2026-07-30

First release of the Oxiland safe Rust core model.

### Added

- `World` feature registry and RAII lifecycle
- `Model` in-memory RDF dataset with default and named-graph CRUD
- Streaming `Model::find` via `StatementMatches` (ADR-005)
- Basic SPARQL `Query` execution (ASK/SELECT) with parse vs evaluation errors
- Oxigraph term re-exports plus `terms::named_node` / `terms::blank_node` helpers (ADR-004)
- Fjall persistence via `Model::open`
- 0.1 compatibility inventory, public API snapshot, and CI/release workflows

### Compatibility claims

- Provides the trusted core model surface described in `docs/reports/0.1.md`
- Does **not** claim full Redland API accounting, differential behavioral
  parity, C source compatibility, or C ABI compatibility
