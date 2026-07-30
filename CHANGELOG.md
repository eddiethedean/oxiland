# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Project charter defining users, scope, non-goals, invariants, success
  measures, and the 1.0 boundary
- Detailed 0.2 RDF I/O milestone plan with decision gates, work packages,
  acceptance matrix, and exit checklist
- Contribution workflow and automated repository-local documentation link check

### Changed

- Aligned roadmap, execution, architecture, parity, verification, decisions,
  risks, and historical release documentation
- Expanded the risk register with status, early signals, contingencies, and 0.2
  parser/format/persistence risks

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
