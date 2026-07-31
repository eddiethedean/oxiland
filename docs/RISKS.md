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
| R-001 | Oxigraph semantics differ from Redland in edge cases | H | H | `active` | Compatibility | native oracle and normalized differential fixtures | first unexplained mismatch | add private adapter or publish reviewed deviation |
| R-002 | “100% parity” becomes an unqualified claim | H | H | `active` | Documentation | chartered claim levels and scoped reports | release text omits inventory/platform/evidence scope | block or correct release communication |
| R-003 | Direct Oxigraph term re-exports prevent a compatibility fix | M | H | `monitoring` | Safe API | ADR-004 trigger, API snapshot, fixture-first review | required behavior cannot be represented | wrapper migration with 0.x migration guide |
| R-004 | Redland C ownership is reproduced unsafely | M | H | `monitoring` | C ABI | separate crate, handle invariants, sanitizers | first pointer or callback API | stop ABI expansion and redesign handle contract |
| R-005 | Legacy storage plug-ins cannot be reproduced | H | M | `monitoring` | Storage | per-backend disposition and migration paths | backend inventory review finds hard dependency | publish capability error or optional integration decision |
| R-006 | Native Redland oracle depends on platform or build versions | M | H | `active` | Compatibility | pin and checksum sources/build metadata | fixture differs across oracle builds | separate platform profiles and investigate before accepting |
| R-007 | Facades materialize unbounded input or output | M | H | `active` | Safe API | streaming core, early-stop and memory gates | large fixture exceeds budget or API returns full collection | redesign before stabilizing affected facade |
| R-008 | Oxigraph upgrade changes behavior, features, or MSRV | M | H | `active` | Safe API | exact release pin and full-suite upgrade gate | dependency update or security advisory | hold upgrade, adapt privately, or revise supported matrix |
| R-009 | C ABI freezes before Rust semantics stabilize | M | H | `monitoring` | C ABI | 0.6 accounting gate before 0.8 | ABI work starts with open mappings | keep symbols experimental or defer ABI milestone |
| R-010 | Upstream conformance claims hide facade defects | M | H | `active` | RDF/SPARQL | run manifests through public Oxiland APIs | Oxigraph-only evidence is proposed | reject verification state until facade suite runs |
| R-011 | Required CI becomes too slow or costly | M | M | `monitoring` | Tooling | tier fast PR, release, and scheduled suites | median PR signal exceeds agreed budget | shard/cache suites without weakening release gates |
| R-012 | Accepted deviations accumulate without review | M | H | `active` | Compatibility | owner, impact, workaround, review milestone | review milestone passes | reopen affected claim and block release if material |
| R-013 | `unsafe`, callback re-entry, or panic creates FFI defects | M | H | `monitoring` | C ABI | local safety proofs, panic boundaries, re-entry tests | C callback support lands | quarantine affected symbol and run focused audit |
| R-014 | Downstream consumers depend on undocumented quirks | H | M | `monitoring` | Compatibility | select consumers early and capture fixtures | first downstream build/test failure | classify quirk and add adapter or limitation |
| R-015 | Packaged artifacts differ from workspace builds | M | H | `active` | Tooling | package dry-run and clean-install tests | packaged smoke test diverges | block publish and repair package manifest/workflow |
| R-016 | 0.x persistent data becomes unreadable or partially updated | M | H | `active` | Storage | label format unstable, export guidance, reopen/failure tests | format change, failed Fjall write, or upgrade request | preserve old reader/export tool; block destructive migration |
| R-017 | Parser failure leaves a model partially loaded unexpectedly | H | H | `monitoring` | RDF/SPARQL | ADR-007 progressive vs collecting APIs | callers assume silent atomicity | document path; add 0.4 transactions |
| R-018 | Format aliases or auto-detection select the wrong syntax | M | M | `monitoring` | RDF/SPARQL | closed Syntax table, ADR-008 | alias collision or misleading extension/MIME | require explicit format and publish corrected mapping |
| R-019 | Planning outruns implementation and evidence | M | H | `active` | Documentation | separate charter/roadmap/milestone/parity authority | planned feature described as available | correct claim and add consistency check/review gate |

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

## Current 0.7 focus

Python package design (ADR-017) against the frozen 0.6 facade. Safe-API
accounting from 0.6 remains a monitoring item. Stream, utility, and logging
risks for the curated 0.5 slice were addressed under ADR-013–ADR-016 and remain
monitoring items. Storage/transaction risks under ADR-006 were addressed in 0.4
(format v1 + migrate). Query/result risks R-001, R-006, R-007, R-008, R-010, and
R-012 were addressed for the 0.3 facade slice and remain monitoring items.
R-017 and R-018 remain monitoring items under ADR-007 and ADR-008. R-003 remains
monitored under ADR-004. C-specific risks remain monitored until their
workstream begins.
