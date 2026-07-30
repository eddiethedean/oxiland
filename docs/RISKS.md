# Risk register

Status: active  
Review: each milestone boundary and before dependency upgrades

Likelihood and impact use `low`, `medium`, or `high`. An owner is a workstream
until individual maintainers are assigned.

| ID | Risk | Likelihood | Impact | Owner | Mitigation | Trigger |
|---|---|---|---|---|---|---|
| R-001 | Oxigraph semantics differ from Redland in edge cases | High | High | Compatibility | differential fixtures and private adapters | mismatch in oracle run |
| R-002 | “100% parity” is interpreted as an unqualified claim | High | High | Documentation | claim levels and scoped reports | public release language omits scope |
| R-003 | Direct Oxigraph re-exports prevent later compatibility fixes | Medium | High | Safe API | decide before 0.2; snapshot public API | wrapper becomes necessary |
| R-004 | Redland C ownership is reproduced unsafely | Medium | High | C ABI | isolate crate, handle invariants, sanitizers | first pointer/callback API |
| R-005 | Legacy storage plug-ins cannot be reproduced | High | Medium | Storage | per-backend decisions and migration tools | backend inventory review |
| R-006 | Native Redland oracle is platform/version dependent | Medium | High | Compatibility | pin/checksum builds and metadata | cross-platform diff disagreement |
| R-007 | Eager APIs cause unbounded memory use | Medium | Medium | Safe API | streaming-by-default gates | large fixture exceeds budget |
| R-008 | Oxigraph upgrade changes behavior or MSRV | Medium | High | Safe API | pin versions; full suite before upgrades | dependency update proposed |
| R-009 | ABI surface freezes before semantics stabilize | Medium | High | C ABI | begin C ABI only after 0.6 | 0.7 work starts with open mappings |
| R-010 | Upstream conformance claims hide facade defects | Medium | High | RDF/SPARQL | run manifests through Oxiland API | Oxigraph-only evidence proposed |
| R-011 | Test matrix becomes too slow to gate releases | Medium | Medium | Tooling | fast PR set plus required release/nightly sets | required CI exceeds target latency |
| R-012 | Accepted deviations accumulate indefinitely | Medium | High | Compatibility | owner and review milestone required | deviation misses review date |
| R-013 | `unsafe` or callbacks allow panic/re-entry bugs | Medium | High | C ABI | panic boundaries, re-entry tests, local safety proofs | callback support lands |
| R-014 | Downstream consumers depend on undocumented quirks | High | Medium | Compatibility | select consumers early and capture fixtures | first downstream test failure |
| R-015 | Platform packaging diverges from tested workspace builds | Medium | High | Tooling | clean-install artifact tests | packaging introduced |

## Escalation rules

A risk blocks a release when:

- its impact is high and its trigger has occurred without a tested mitigation;
- it can invalidate the milestone's compatibility claim;
- it can cause data loss, memory unsafety, or ABI corruption;
- its mitigation depends on an unresolved architecture decision.

Closing a risk requires evidence that the trigger is removed or the mitigation
is operating. Rewording the risk or moving it to known limitations is not
closure.

## Current focus

For 0.2, R-001, R-008, and R-010 remain active. R-003 and R-007 are mitigated
for the 0.1 surface by ADR-004/ADR-005 and the public-API snapshot, but stay
monitored for later facades. R-004, R-009, R-013, and R-015 remain monitored
until their workstreams begin.

