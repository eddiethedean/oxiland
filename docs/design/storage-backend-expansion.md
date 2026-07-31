# Storage backend expansion plan

Status: stabilization in progress
Targets: 0.8 design, 0.9 adapters, 0.10 stabilization
Decisions: ADR-022 and ADR-024
Existing contract: ADR-006 / Fjall format v1

## Outcome

Oxiland will keep `Model` and its RDF/SPARQL behavior independent from the
durable key-value engine. Fjall remains the compatible default for
`Model::open(path)`, while applications may explicitly select another compiled
backend through `OpenOptions`.

The expansion does not make native database directories interchangeable.
Standards RDF remains the cross-backend archival and migration contract.

## Backend selection criteria

A backend fits the first-party plan only when its Rust integration can provide:

- embedded, process-local operation with no required database server;
- byte keys and values plus complete key iteration;
- an atomic multi-key write batch suitable for committing one Oxiland change;
- an explicit durability barrier (`sync`, `flush`, or durable commit);
- deterministic create/open behavior and a way to reject the wrong layout;
- crash recovery to either the pre-commit or post-commit state;
- licenses compatible with Oxiland and a supportable MSRV/platform matrix.

Read-only open, multi-process access, snapshots, compression, and online backup
are reported as capabilities. They are not silently emulated when an engine
cannot provide them.

Popularity is an inclusion signal, not an acceptance gate by itself. The list
below covers the established embedded engines most likely to be requested by
Rust users while keeping server databases outside the local-store contract.

## Planned backend matrix

| Backend name | Engine / integration | Plan | Why it fits | Promotion gate |
|---|---|---|---|---|
| `memory` | Oxigraph in-memory store | Built-in non-durable baseline | Existing zero-configuration model and transaction behavior | Keep capability reporting explicit; no path or durability claims |
| `fjall` | [Fjall](https://github.com/fjall-rs/fjall) | Built-in default; conformance baseline | Existing pure-Rust LSM backend and format-v1 implementation | Preserve 0.4–0.7 reopen behavior while moving behind the adapter |
| `redb` | [redb](https://github.com/cberner/redb) | First-party optional adapter | Pure Rust, stable format, ACID transactions, MVCC, crash-safe commits | MSRV, read-only-open behavior, file locking, and full conformance matrix |
| `rocksdb` | [RocksDB](https://github.com/facebook/rocksdb) via `rust-rocksdb` | First-party optional adapter | Widely deployed embedded LSM with batches, transactions, checkpoints, and large-store tuning | Optional C++ toolchain, license/features review, packaged builds, and read-only tests |
| `sqlite` | [SQLite](https://sqlite.org/transactional.html) via `rusqlite` | First-party optional adapter | Portable single-file ACID store; byte-key table is easy to inspect and operate | Fixed schema/PRAGMAs, WAL/rollback-mode crash tests, locking, and bundled/system build policy |
| `lmdb` | [LMDB](https://github.com/LMDB/lmdb) via [heed](https://github.com/meilisearch/heed) | First-party optional adapter | Mature memory-mapped transactional key-value engine with cheap readers | Map-size configuration, safe environment-open wrapper, platform packaging, and writer-lock tests |
| `sled` | [sled](https://github.com/spacejam/sled) | Gated optional evaluation | Popular pure-Rust API with transactions, batches, ordered iteration, and flush | Do not advertise as supported while upstream describes it as beta with a changing on-disk format; require a migration and maintenance decision |
| `leveldb` | [LevelDB](https://github.com/google/leveldb) through a selected maintained Rust binding | Gated optional evaluation | Established ordered key-value engine with snapshots and atomic write batches | Binding selection, native-build policy, read-only/capability gaps, and crash/reopen evidence |
| `mdbx` | [libmdbx](https://github.com/erthink/libmdbx) through a selected maintained Rust binding | Gated optional evaluation | Transactional memory-mapped B+tree and useful LMDB-family alternative | Binding/API maturity, source distribution, platform support, and license review |
| `surrealkv` | [SurrealKV](https://github.com/surrealdb/surrealkv) | Gated optional evaluation | Pure-Rust embedded transactional LSM with versioned reads | General-purpose API stability, independent maintenance expectations, format policy, and conformance evidence |

“First-party optional” is planned scope, not a claim that the adapter is
currently shipped. A gated evaluation gets a canonical name and spike in the
plan, but it is promoted only when the same acceptance suite as Fjall passes.
An engine that fails a gate stays explicitly unavailable rather than receiving
a partial adapter.

Remote or service-backed stores such as Redis, FoundationDB, TiKV, PostgreSQL,
and cloud object stores do not fit this path-based synchronous contract. They
would require a separate remote/async model contract. Historical Redland
storage names also remain governed by the per-backend compatibility decision;
for example, the new `sqlite` backend would be an Oxiland byte-key layout, not
binary compatibility with a Redland SQLite plug-in.

## Adapter boundary

0.8 extracts the Fjall-specific code behind a private, sealed durable-store
adapter. The minimum semantic operations are:

- identify the backend and validate its layout marker;
- initialize and read Oxiland format metadata;
- scan every persisted quad key;
- atomically apply a set of puts and deletes;
- force all acknowledged changes to stable storage;
- report capabilities and engine-specific diagnostics.

The adapter owns physical database calls; `Model` owns RDF equality, locking,
the Oxigraph working set, transaction behavior, and error translation. No
engine type appears in the default public facade.

The boundary remains sealed for 1.0. ADR-024 rejected exposing a
user-supplied `DurableBackend` trait after reviewing object safety, thread
safety, panic containment, re-entry, crash atomicity, registry consistency,
and SemVer. A future provider API needs an external conformance/failure-
injection boundary rather than exposing the first-party physical adapter.

## Public selection surface

The planned API shape is:

- preserve `Model::open(path)` as the Fjall-compatible shortcut;
- add `OpenOptions::new(StorageBackend, path)` plus named convenience
  constructors for first-party adapters;
- keep canonical `StorageBackend` identities recognizable in every build;
- return a specific `Unsupported` error when a known backend was not compiled;
- reserve Cargo features `storage-fjall`, `storage-redb`, `storage-rocksdb`,
  `storage-sqlite`, and `storage-lmdb`, with native engines off by default;
- expose backend availability and capabilities through Rust, the CLI, Python,
  and the future C ABI from one registry;
- reject opening a path whose marker names a different backend.

Evaluation-only feature names are added only when their promotion ADR is
accepted. Disabling an adapter must not make its canonical name look unknown.

## Format and migration policy

- Existing Fjall format-v1 directories remain readable without an implicit
  rewrite.
- Each new engine gets a versioned physical layout and a marker containing the
  canonical backend identity. A format number never implies that two engines'
  native files are interchangeable.
- N-Quads/TriG export and transactional import are the required migration path
  for every backend.
- 0.9 evaluates a streaming `Model::copy_to(OpenOptions)` helper. It must use a
  newly created destination and leave a failed destination identifiable and
  safe to remove; it must not rewrite the source in place.
- Backend removal requires a readable export window and release notes. A
  dependency may not disappear while it is the only reader for a supported
  layout.

## Verification matrix

One backend-conformance harness runs against every promoted adapter:

1. create/open/create-false/wrong-backend and read-only behavior;
2. insert, delete, clear, named-graph clear, bulk load, and full iteration;
3. transaction commit and rollback with atomic durable visibility;
4. explicit sync followed by process restart;
5. injected failures before, during, and after batch commit;
6. concurrent readers and the engine's documented writer/multi-process model;
7. format metadata corruption, unsupported version, and migration/export;
8. bounded large-store performance and disk-amplification baselines;
9. supported target, MSRV, minimal-feature, and packaged-artifact builds.

Adapter-specific limitations are expressed in `StorageCapabilities` and tested.
The common `Model::transaction` contract cannot be weakened for one backend.

## Work packages

| Target | Work package | Exit evidence |
|---|---|---|
| 0.8 | SB-01 contract and ADR | ADR-022 accepted; exact adapter invariants and public selection API reviewed |
| 0.8 | SB-02 Fjall extraction | Existing format-v1 suite passes through the sealed adapter |
| 0.8 | SB-03 conformance harness | One reusable suite covers memory/Fjall and can register optional adapters |
| 0.9 | SB-04 pure-Rust alternative | `redb` adapter and feature/package tests pass |
| 0.9 | SB-05 established native alternatives | RocksDB, SQLite, and LMDB adapters pass their platform matrices |
| 0.9 | SB-06 gated evaluations | sled, LevelDB, MDBX, and SurrealKV each receive a promote/defer/reject record with evidence |
| 0.9 | SB-07 user surfaces and migration | Rust, CLI, Python, C capability discovery, docs, and cross-backend copy/export tests agree |
| 0.10 | SB-08 stabilization, Redland parity, and performance | ADR-024 freezes six identities/features and the format-v1 reader/export policy; API/ABI snapshots, crash matrix, dependency audit, full baseline storage-factory behavior, and required benchmark wins remain release-qualification gates; capability errors and migration-only paths do not count |

## Non-goals

- Selecting a backend automatically from performance guesses.
- Silently mapping one backend name to another engine.
- Enabling every native dependency in the default build.
- Promising byte-for-byte native directory portability.
- Treating a backend benchmark as proof of RDF query performance; Oxigraph
  still holds the complete in-memory query working set under the current model.
