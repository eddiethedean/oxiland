# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
