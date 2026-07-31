# Risk register

Status: active  
Review: when a trigger occurs, after each completed work package, at milestone
boundaries, and before dependency upgrades

Likelihood and impact use `low`, `medium`, or `high`. Status is `monitoring`,
`active`, `mitigated`, or `closed`. An owner is a workstream until individual
maintainers are assigned.

`mitigated` means a tested control reduces exposure; it does not mean the risk
can never recur. `closed` requires the underlying exposure to be removed.

## Register

| ID | Risk | L | I | Status | Owner | Preventive response | Trigger / early signal | Contingency |
|---|---|:---:|:---:|---|---|---|---|---|
| R-001 | Oxigraph semantics differ from Redland in edge cases | H | H | `active` | Compatibility | native oracle and normalized differential fixtures | first unexplained mismatch | add a private adapter; any remaining deviation blocks 0.10 |
| R-002 | “100% parity” becomes an unqualified claim | H | H | `active` | Documentation | chartered claim levels and scoped reports | release text omits inventory/platform/evidence scope | block or correct release communication |
| R-003 | Direct Oxigraph term re-exports prevent a compatibility fix | M | H | `monitoring` | Safe API | ADR-004 trigger, API snapshot, fixture-first review | required behavior cannot be represented | wrapper migration with 0.x migration guide |
| R-004 | Redland C ownership is reproduced unsafely | M | H | `monitoring` | C ABI | separate crate, handle invariants, sanitizers | first pointer or callback API | stop ABI expansion and redesign handle contract |
| R-005 | Baseline storage behavior cannot be reproduced | H | H | `monitoring` | Storage | pin canonical baseline profiles and test each factory behavior | backend inventory review finds hard dependency | implement an equivalent adapter or hold 0.10; capability errors and migration-only paths do not satisfy parity |
| R-006 | Native Redland oracle depends on platform or build versions | M | H | `active` | Compatibility | pin and checksum sources/build metadata | fixture differs across oracle builds | separate platform profiles and investigate before accepting |
| R-007 | Facades materialize unbounded input or output | M | H | `active` | Safe API | streaming core, early-stop and memory gates | large fixture exceeds budget or API returns full collection | redesign before stabilizing affected facade |
| R-008 | Oxigraph upgrade changes behavior, features, or MSRV | M | H | `active` | Safe API | exact release pin and full-suite upgrade gate | dependency update or security advisory | hold upgrade, adapt privately, or revise supported matrix |
| R-009 | C ABI freezes before Rust semantics stabilize | M | H | `monitoring` | C ABI | 0.6 accounting gate before 0.8 | ABI work starts with open mappings | keep symbols experimental or defer ABI milestone |
| R-010 | Upstream conformance claims hide facade defects | M | H | `active` | RDF/SPARQL | run manifests through public Oxiland APIs | Oxigraph-only evidence is proposed | reject verification state until facade suite runs |
| R-011 | Required CI becomes too slow or costly | M | M | `monitoring` | Tooling | tier fast PR, release, and scheduled suites | median PR signal exceeds agreed budget | shard/cache suites without weakening release gates |
| R-012 | Accepted deviations accumulate before the parity freeze | M | H | `active` | Compatibility | owner, impact, workaround, review milestone | review milestone passes | reopen the affected claim; every in-scope deviation blocks 0.10 |
| R-013 | `unsafe`, callback re-entry, or panic creates FFI defects | M | H | `monitoring` | C ABI | local safety proofs, panic boundaries, re-entry tests | C callback support lands | quarantine affected symbol and run focused audit |
| R-014 | Downstream consumers depend on undocumented quirks | H | M | `monitoring` | Compatibility | select consumers early and capture fixtures | first downstream build/test failure | classify quirk and add adapter or limitation |
| R-015 | Packaged artifacts differ from workspace builds | M | H | `active` | Tooling | package dry-run and clean-install tests | packaged smoke test diverges | block publish and repair package manifest/workflow |
| R-016 | 0.x persistent data becomes unreadable or partially updated | M | H | `active` | Storage | label format unstable, export guidance, reopen/failure tests | format change, failed Fjall write, or upgrade request | preserve old reader/export tool; block destructive migration |
| R-017 | Parser failure leaves a model partially loaded unexpectedly | H | H | `monitoring` | RDF/SPARQL | ADR-007 progressive vs collecting APIs | callers assume silent atomicity | document path; add 0.4 transactions |
| R-018 | Format aliases or auto-detection select the wrong syntax | M | M | `monitoring` | RDF/SPARQL | closed Syntax table, ADR-008 | alias collision or misleading extension/MIME | require explicit format and publish corrected mapping |
| R-019 | Planning outruns implementation and evidence | M | H | `active` | Documentation | separate charter/roadmap/milestone/parity authority | planned feature described as available | correct claim and add consistency check/review gate |
| R-020 | Oxigraph 0.5.9 pins vulnerable `quick-xml` 0.37 in RDF/XML and SPARQL XML paths | M | H | `active` | Safe API | Tip/release CI share a narrow, self-expiring ignore for RUSTSEC-2026-0194/0195 verified by `scripts/check-security-exceptions.py` | Oxigraph releases a line that accepts `quick-xml` 0.41+ (main already uses 0.41) | upgrade Oxigraph under the full compatibility suite; remove the tip ignores |
| R-021 | Optional storage engines weaken durability or strand backend-specific data | M | H | `active` | Storage | ADR-022, sealed adapter, shared conformance/crash matrix, versioned layout markers, RDF export path | first non-Fjall adapter, dependency removal, wrong-backend open, or divergent transaction result | do not promote/freeze the adapter; preserve its reader/export feature and migrate through standards RDF |
| R-022 | “Faster than Redland” is produced by noise, cherry-picked workloads, or unequal builds | H | H | `active` | Performance | freeze representative workloads and matched-build protocol; publish raw samples, ratios, and confidence intervals | any tie/loss, unstable result, deleted case, or environment mismatch | repair performance or measurement validity and rerun the full matrix; hold 0.10 |
| R-023 | Optional LMDB dependency `heed` retains unmaintained `bincode` 1.3.3 | M | M | `active` | Storage | RustSec audit on all lockfiles; track `heed` dependency updates; keep LMDB optional | a vulnerability, format defect, or unsupported-toolchain issue lands in the inherited crate | update `heed` under the storage conformance suite or preserve the LMDB reader/export window while replacing the adapter dependency |

## Release-blocking rule

A risk blocks a release when any of these is true:

- impact is high and its trigger has occurred without a tested response;
- it invalidates the milestone outcome or a published compatibility claim;
- it can cause data loss, memory unsafety, silent semantic corruption, or ABI
  corruption;
- its response depends on an unresolved decision required by the milestone;
- the release would make the risky behavior harder to change or recover from.

The release report lists all active high-impact risks and explains why each is
controlled or blocking.

## Review and status changes

At review:

1. confirm likelihood and impact against current implementation;
2. inspect triggers and linked evidence;
3. verify the preventive response is actually running;
4. create or update the contingency owner and next action;
5. change status only with a reason and evidence link.

Closing a risk requires evidence that the exposure is gone. Renaming it as a
known limitation is not closure. A regression may move a mitigated or closed
risk back to active.

## Current 0.10 focus

The hard parity and performance gates make R-001, R-002, R-005, R-006,
R-012, R-015, R-019, and R-022 release-blocking until complete evidence is
checked in. `oxiland-capi` risks R-004, R-009, and R-013 remain active while
the 0.9 exclusions are replaced by verified implementations. ADR-024 freezes
the first-party backend registry and reader/export policy, but R-021 remains
active until the full cross-platform crash and packaged-reader matrix passes.
Python wheel and public-surface snapshots remain qualification inputs.

R-020 remains active: PyO3 is 0.29.0, but Oxigraph 0.5.9 still constrains
`quick-xml` 0.37. The narrow CI exception remains self-expiring and blocks any
unreviewed dependency-graph change. R-023 records cargo-audit's maintenance
warning for `bincode` through the optional LMDB adapter. No active high-impact
risk is considered waived merely because its preventive validator exists.
