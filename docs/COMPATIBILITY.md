# Compatibility contract

Status: active contract  
Normative ledger: [`parity.md`](parity.md) (canonical root file: [`PARITY.md`](https://github.com/eddiethedean/oxiland/blob/main/PARITY.md))  
Verification method: [`VERIFICATION.md`](VERIFICATION.md)

## Baseline and target

The baseline is Redland `librdf` 1.0.17 and the reference manual labeled
1.0.18. Raptor and Rasqal behavior is in scope when exposed through a public
`librdf` operation. Their complete independent APIs are not automatically in
scope.

Compatibility has distinct claims:

- **Concept parity:** Redland workflows have idiomatic Rust equivalents.
- **Safe API accounting:** every public Redland item is mapped or classified.
- **Source compatibility:** supported C programs compile against Oxiland's
  headers.
- **ABI compatibility:** existing binaries can load Oxiland without
  recompilation.
- **Behavioral compatibility:** equivalent calls produce equivalent observable
  results and failures.

No document or release may use “100% parity” without naming which claim it
means. The 1.0 aspiration is safe API accounting plus the published C
source/ABI and behavioral compatibility surface.

## Claim levels

| Level | Required evidence | Earliest milestone |
|---|---|---|
| Concept parity | workflow mapping and examples | 0.2 |
| Safe API accounting | complete inventory classification | 0.6 |
| Safe behavioral parity | differential fixtures for mapped behavior | 0.6 |
| Python package usability | Pythonic API + installable wheels + pytest | 0.7 |
| C source compatibility | clean builds against Oxiland headers | 0.8 |
| C ABI compatibility | symbol, layout, calling, and lifecycle tests | 0.9 |
| Downstream compatibility | selected real consumers pass unchanged | 0.9 |

Claims are subsystem- and platform-scoped until 1.0. For example, “parser
behavior verified on Linux” does not imply full storage ABI compatibility on
Windows.

## Canonical inputs

The inventory is derived from version-pinned copies of:

- installed/public Redland headers;
- Redland reference documentation;
- exported symbols from the reference shared library;
- representative native behavior captured by the oracle runner.

The source version, build configuration, operating system, Raptor version, and
Rasqal version are recorded with generated evidence. Generated inputs are
checksummed so a changed oracle cannot silently rewrite expectations.

## Inventory schema

The compatibility inventory will be generated from the canonical Redland
headers and enriched with documentation metadata. Each entry records:

- stable ID, symbol, kind, header, and normalized C signature;
- subsystem and lifecycle/ownership rules;
- safe Rust mapping (`safe_rust`);
- C ABI fields (required from milestone **0.8** onward):
  - `c_abi` — Oxiland C export name when implemented, or `null` when not
    exported in this milestone;
  - `c_state` — C claim state using the same allowed states as `state`, scoped
    to source-compat / ABI work (`unreviewed` until a C disposition exists);
  - `c_tests` — optional list of CAPI or sanitizer test references when
    `c_state` is `implemented` or `verified`;
- support status, platform, and feature gate;
- behavioral test identifiers;
- deviations, rationale, and evidence links.

For milestones before 0.8, `c_abi` / `c_state` may be omitted (safe-API
accounting only). The 0.8 inventory must populate them for every entry: preview
allowlist symbols reach `verified` or `implemented` with `c_tests`; remaining
symbols are `mapped` (deferred to 0.9), `not-applicable`, or `excluded` with
notes.

Allowed states are `unreviewed`, `mapped`, `implemented`, `verified`,
`not-applicable`, and `excluded`. `not-applicable` is reserved for mechanics
replaced by Rust ownership. `excluded` requires a written compatibility impact
assessment.

State transitions are monotonic except when a regression reopens an item:

```text
unreviewed -> mapped -> implemented -> verified
                  ├──> not-applicable
                  └──> excluded
```

“Implemented” means code exists. Only “verified” contributes to behavioral
parity metrics.

## Behavioral contract

Tests compare more than successful return values. They cover:

- term equality and canonical string forms;
- duplicate statement behavior;
- blank-node identity;
- language and datatype handling;
- context/default-graph semantics;
- parser recovery and diagnostics;
- serialization and namespace behavior;
- query result types, ordering where guaranteed, and errors;
- storage persistence and transaction boundaries;
- callback ordering, logging, and lifecycle edge cases.

Output comparison uses semantic normalization where formats permit irrelevant
variation. For example, RDF graphs compare as datasets rather than raw Turtle
bytes unless byte formatting itself is the tested contract.

## Normalization rules

Comparison must not erase meaningful incompatibilities:

- RDF graph and dataset order is ignored; duplicate semantics are preserved.
- Blank nodes are compared by graph isomorphism, not source labels.
- Query bindings retain variable names, unbound values, datatypes, and language.
- Serialized bytes are normalized only for tests about RDF meaning; formatting
  tests compare bytes or tokens directly.
- Diagnostics compare category and structured location first; exact prose is
  required only when a consumer-facing contract depends on it.
- File paths, temporary directories, and allocator addresses may be redacted.

Each fixture names its normalization profile.

## Ownership and error parity

For safe Rust, manual allocation calls may be `not-applicable`, but their
observable effects—cloning, aliasing, invalidation, and lifetime—remain part of
the mapped object's contract. For the C ABI, allocator pairing and pointer
lifetime are behavioral requirements.

Redland integer/null return conventions map to typed Rust results. The C shim
maps them back exactly where source/ABI compatibility is claimed. Extra Rust
diagnostic detail is allowed provided callers can still classify the original
failure.

## Storage compatibility

Historical MySQL, PostgreSQL, SQLite, TStore, URI, and Virtuoso plug-ins cannot
be assumed equivalent to Oxigraph storage. Each receives an individual
decision:

1. native adapter with equivalent behavior;
2. migration/import tooling;
3. compatibility error identifying the unsupported backend; or
4. a separately maintained optional integration.

Backend names must never silently select a different persistence technology.

## Platform and feature scope

Every compatibility report identifies:

- target triple and linker/ABI;
- Oxiland and Oxigraph versions;
- enabled features;
- storage backend;
- native Redland oracle build;
- test suite revision.

Feature-disabled APIs may return a specific capability error. They may not
disappear from a promised C ABI without a separately named reduced artifact.

## Deviations and exclusions

An accepted deviation records:

- affected inventory IDs and workflows;
- observable difference and user impact;
- reason exact behavior is unsafe, impossible, or disproportionate;
- migration or workaround;
- owner and next review milestone;
- whether it blocks a named compatibility claim.

Exclusions require review before each release candidate. Convenience is not a
sufficient rationale for excluding a public Redland behavior.

## Change control

Every pull request affecting compatibility updates at least one of:

- the inventory;
- differential fixtures;
- the parity ledger;
- compatibility notes.

A Redland deviation is a release-note item. Fixing a deviation may itself be a
breaking behavioral change during 0.x and must include a migration note.

Regressions move affected inventory items from `verified` back to
`implemented`, retain prior evidence for audit history, and block release gates
for the subsystem's current claim.
