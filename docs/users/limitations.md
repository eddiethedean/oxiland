# Known limitations

This page indexes intentional product boundaries. Prefer the linked guide for
detail; do not treat silence elsewhere as support.

## Persistence and process model

- Oxiland is an **embedded** library, not a network database, multi-tenant
  server, or managed backup service.
  ([FAQ](faq.md), [Rust production](rust-production.md),
  [Python production](python-production.md))
- Persistent (Fjall) models keep a full in-memory Oxigraph working set for
  query—plan RAM for the dataset size.
  ([Performance](performance.md))
- Do not share one writable store directory across mutually untrusted processes
  as a coordination protocol. ([FAQ](faq.md))
- Format v1 reopens across **0.4.x–0.12.x** patch lines; export N-Quads before
  any future format-v2 migration. ([Support policy](../support.md))

## RDF I/O

- Unknown or ambiguous formats raise `Unsupported` / `UnsupportedError`; Oxiland
  does not guess syntax from document contents. ([FAQ](faq.md), [I/O](io.md))
- The default graph target **rejects** named-graph input. Rust can use
  `GraphTarget::Dataset`; Python does not expose that target today—use
  `graph=` or programmatic named-graph CRUD.
  ([Python data](python-data.md), [FAQ](faq.md))
- Progressive loads can leave partial data on failure; use transactional /
  collecting load when atomic import is required. ([FAQ](faq.md))

## CLI

- Memory mode is one-shot for multi-step workflows that expect a durable path.
- Dataset-style N-Quads/TriG import has the same named-graph target limitation
  as the default graph load path; restore multi-graph backups with Rust/Python
  `import_nquads` APIs. ([CLI guide](cli.md))

## C ABI

- `oxiland-capi` is `publish = false` (build from a repository checkout).
- Remaining fail-closed behavioral gaps (stream maps, namespace tracking,
  serializer/factory callbacks, and related limits) are listed in
  [C ABI limitations](c-abi-limitations.md).

## Optional storage backends

- Backend selection fails closed: unknown names and known-but-not-compiled
  optional backends never silently fall back to Fjall or memory.
  ([FAQ](faq.md), [Persistence](persistence.md))
- Physical layouts are not interchangeable; migrate with standards RDF or
  `Model::copy_to`.

## Python package

- Not an rdflib adapter and not layered on `oxiland-capi`.
- Published packages are wheels only (no sdist on PyPI).
  ([Python installation](python-installation.md))

## Performance claims

- Competitive parity (ADR-028) and suite-wide faster-than-Redland (ADR-029)
  are distinct claim classes; both are closed on tip evidence. Host-scoped
  highlight tables remain separately scoped when cited alone.
  ([Performance claims policy](performance.md#claims-policy))
