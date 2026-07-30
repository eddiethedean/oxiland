# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- User and evaluator documentation paths under `docs/users/` and
  `docs/evaluators/`
- `SECURITY.md` and `CODE_OF_CONDUCT.md`
- Examples `select` and `progressive_load`
- Milestone stub `docs/milestones/0.3.md`
- MkDocs / Read the Docs site (`.readthedocs.yaml`, `mkdocs.yml`)

### Changed

- crates.io description and categories no longer overclaim Redland compatibility
- README rewritten for adoption (when-to-use, experimental Fjall, format table,
  role-based doc links, badges)
- Docs index is a Users / Evaluators / Contributors router
- Parity ledger clarifies scoped meaning of `verified`
- `Parser::parse_path_with_extension` uses `GraphTarget::Dataset` for N-Quads
  and TriG
- `GraphTarget::Named` keeps quads already named for the target graph (rejects
  only foreign named graphs)
- Progressive `load_into` annotates partial progress on I/O and storage errors,
  not only parse failures

### Fixed

- Fjall duplicate-insert disk failure no longer removes a pre-existing in-memory
  statement
- Concurrent `add` / `remove` return values are serialized so both callers cannot
  observe `true` for a single insert/remove
- Public API snapshot CI verifies owned public items against the baseline
- Inventory checker verifies cited test function names exist

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
