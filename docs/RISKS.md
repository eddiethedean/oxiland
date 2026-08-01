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
| R-001 | Oxigraph semantics differ from Redland in edge cases | H | H | `mitigated` | Compatibility | native oracle and normalized differential fixtures; 0.10 inventory + parity evidence | first unexplained mismatch | add a private adapter; any remaining deviation blocks 0.10 |
| R-002 | “100% parity” becomes an unqualified claim | H | H | `mitigated` | Documentation | chartered claim levels, scoped reports, `check-0.10-parity.py` / release gate | release text omits inventory/platform/evidence scope | block or correct release communication |
| R-003 | Direct Oxigraph term re-exports prevent a compatibility fix | M | H | `monitoring` | Safe API | ADR-004 trigger, API snapshot, fixture-first review | required behavior cannot be represented | wrapper migration with 0.x migration guide |
| R-004 | Redland C ownership is reproduced unsafely | M | H | `mitigated` | C ABI | separate crate, handle invariants, ASan/UBSan, full C surface lifecycle tests | first pointer or callback API | stop ABI expansion and redesign handle contract |
| R-005 | Baseline storage behavior cannot be reproduced | H | H | `mitigated` | Storage | pin canonical baseline profiles; factory facade; crash/conformance matrix | backend inventory review finds hard dependency | implement an equivalent adapter or hold 0.10; capability errors and migration-only paths do not satisfy parity |
| R-006 | Native Redland oracle depends on platform or build versions | M | H | `mitigated` | Compatibility | pin and checksum sources/build metadata; frozen 0.10 matrix profiles | fixture differs across oracle builds | separate platform profiles and investigate before accepting |
| R-007 | Facades materialize unbounded input or output | M | H | `active` | Safe API | streaming core, early-stop and memory gates | large fixture exceeds budget or API returns full collection | redesign before stabilizing affected facade |
| R-008 | Oxigraph upgrade changes behavior, features, or MSRV | M | H | `active` | Safe API | exact release pin and full-suite upgrade gate | dependency update or security advisory | hold upgrade, adapt privately, or revise supported matrix |
| R-009 | C ABI freezes before Rust semantics stabilize | M | H | `mitigated` | C ABI | 0.6 accounting gate before 0.8; full C surface verified in 0.10 inventory | ABI work starts with open mappings | keep symbols experimental or defer ABI milestone |
| R-010 | Upstream conformance claims hide facade defects | M | H | `active` | RDF/SPARQL | run manifests through public Oxiland APIs | Oxigraph-only evidence is proposed | reject verification state until facade suite runs |
| R-011 | Required CI becomes too slow or costly | M | M | `monitoring` | Tooling | tier fast PR, release, and scheduled suites | median PR signal exceeds agreed budget | shard/cache suites without weakening release gates |
| R-012 | Accepted deviations accumulate before the parity freeze | M | H | `mitigated` | Compatibility | owner, impact, workaround, review milestone; hard gate rejects deviations | review milestone passes | reopen the affected claim; every in-scope deviation blocks 0.10 |
| R-013 | `unsafe`, callback re-entry, or panic creates FFI defects | M | H | `mitigated` | C ABI | local safety proofs, panic boundaries, re-entry tests, ASan/UBSan, fuzz targets | C callback support lands | quarantine affected symbol and run focused audit |
| R-014 | Downstream consumers depend on undocumented quirks | H | M | `monitoring` | Compatibility | select consumers early and capture fixtures | first downstream build/test failure | classify quirk and add adapter or limitation |
| R-015 | Packaged artifacts differ from workspace builds | M | H | `mitigated` | Tooling | package dry-run and clean-install tests; wheel/C smoke CI | packaged smoke test diverges | block publish and repair package manifest/workflow |
| R-016 | 0.x persistent data becomes unreadable or partially updated | M | H | `active` | Storage | label format unstable, export guidance, reopen/failure tests | format change, failed Fjall write, or upgrade request | preserve old reader/export tool; block destructive migration |
| R-017 | Parser failure leaves a model partially loaded unexpectedly | H | H | `monitoring` | RDF/SPARQL | ADR-007 progressive vs collecting APIs | callers assume silent atomicity | document path; add 0.4 transactions |
| R-018 | Format aliases or auto-detection select the wrong syntax | M | M | `monitoring` | RDF/SPARQL | closed Syntax table, ADR-008 | alias collision or misleading extension/MIME | require explicit format and publish corrected mapping |
| R-019 | Planning outruns implementation and evidence | M | H | `mitigated` | Documentation | separate charter/roadmap/milestone/parity authority; qualification validators | planned feature described as available | correct claim and add consistency check/review gate |
| R-020 | Oxigraph 0.5.9 pins vulnerable `quick-xml` 0.37 in RDF/XML and SPARQL XML paths | M | H | `active` | Safe API | Tip/release CI share a narrow, self-expiring ignore for RUSTSEC-2026-0194/0195 verified by `scripts/check-security-exceptions.py` | Oxigraph releases a line that accepts `quick-xml` 0.41+ (main already uses 0.41) | upgrade Oxigraph under the full compatibility suite; remove the tip ignores |
| R-021 | Optional storage engines weaken durability or strand backend-specific data | M | H | `mitigated` | Storage | ADR-022/024, sealed adapter, shared conformance/crash matrix (`backend_conformance`), versioned layout markers, RDF export path | first non-Fjall adapter, dependency removal, wrong-backend open, or divergent transaction result | do not promote/freeze the adapter; preserve its reader/export feature and migrate through standards RDF |
| R-022 | “Faster than Redland” is produced by noise, cherry-picked workloads, or unequal builds | H | H | `mitigated` | Performance | freeze representative workloads and matched-build protocol; raw-sample validators and performance tooling | any tie/loss, unstable result, deleted case, or environment mismatch | repair performance or measurement validity and rerun the full matrix; hold 0.10 |
| R-023 | Optional LMDB dependency `heed` retains unmaintained `bincode` 1.3.3 | M | M | `mitigated` | Storage | ADR-027: keep heed/bincode for 0.10 with LMDB optional; RustSec audit on lockfiles; track upstream | a vulnerability, format defect, or unsupported-toolchain issue lands in the inherited crate | update `heed` under the storage conformance suite or preserve the LMDB reader/export window while replacing the adapter dependency |

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

Mitigated with checked-in controls (status does not waive residual monitoring):

- R-001 / R-002 / R-006 / R-012 / R-019: inventory
  `compatibility/inventory/redland-1.0.17-oxiland-0.10.json`, parity evidence
  `compatibility/qualification/0.10-parity-evidence.json`, and fail-closed
  validators.
- R-004 / R-009 / R-013: full C ABI surface, lifecycle tests, ASan, and UBSan
  (Linux) on `oxiland-capi`.
- R-005 / R-021: ADR-024 freeze plus `tests/backend_conformance.rs` crash and
  durability matrix.
- R-015: package/wheel/C smoke and clean-install CI paths.
- R-022: frozen suite plus performance gate tooling (raw samples still required
  before a faster-than-Redland claim).
- R-023: ADR-027 accepts optional `heed`/`bincode` for 0.10 while tracking
  upstream.

Still active and release-relevant: R-007, R-008, R-010, R-016, and R-020
(narrow `quick-xml` CI exception). No active high-impact risk is waived merely
because its preventive validator exists; missing performance raw samples still
block `scripts/check-0.10-release.py`.
