# Security policy

Security reports are handled privately. Please do not open a public issue,
discussion, pull request, or proof-of-concept repository for a suspected
vulnerability before coordinated disclosure.

## Supported versions

| Version | Security status |
|---|---|
| 0.7.x | Supported |
| 0.6.x | Supported when practical; upgrade preferred |
| 0.4.x–0.5.x | Critical fixes when practical; upgrade strongly preferred |
| 0.1.x–0.3.x | Security fixes when practical; upgrade strongly preferred |
| Earlier / unreleased snapshots | Unsupported |

Because Oxiland is pre-1.0, a security fix may require a documented breaking
minor release when a safe backport is not possible.

## Report a vulnerability

Email **odosmatthews@gmail.com** with a subject beginning `Oxiland security`.
Include as much of the following as is safe:

- affected Oxiland version, package surface, platform, and architecture;
- vulnerability class and expected impact;
- minimal reproduction or proof of concept;
- whether untrusted RDF, SPARQL, paths, or store contents are required;
- known workarounds or mitigations;
- whether the issue also appears to affect an upstream dependency;
- your preferred attribution and disclosure constraints.

Do not send production credentials, complete private datasets, or an original
live store. Reduce the report to a sanitized fixture where possible.

You should receive acknowledgment within a few business days. Maintainers will
attempt to confirm scope, identify affected versions, coordinate with upstream
projects where necessary, and agree on a disclosure plan. This is a best-effort
open-source process, not a response-time SLA.

## Security scope

Reports are especially relevant when they involve:

- memory safety, panic containment, or unsafe dependency behavior;
- malformed RDF/SPARQL causing crashes, corruption, or uncontrolled resource use;
- persistent-store corruption, rollback failure, or cross-dataset access;
- path handling that escapes an application-authorized boundary;
- dependency or release-artifact compromise;
- future C ABI handle, allocation, callback, or symbol safety.

The primary Rust crate uses `#![forbid(unsafe_code)]`. Native dependencies and
the Python extension still require dependency and boundary review; the absence
of unsafe code in the facade is not a complete security proof.

## Deployment threat model

Oxiland is an embedded library, not a hardened multi-tenant database server.
It does not provide authentication, authorization, network isolation,
replication, encryption at rest, query quotas, or sandboxing.

Applications are responsible for:

- authorizing store, import, export, and backup paths;
- applying operating-system permissions and storage encryption where required;
- limiting untrusted document size, query complexity, result counts, and runtime;
- isolating expensive or hostile workloads;
- protecting and testing backups;
- avoiding sensitive RDF, SPARQL, and literals in logs;
- monitoring disk, memory, latency, cancellation, and concrete failure classes.

Persistent store directories should be treated as trusted local state. Do not
share one writable directory across mutually untrusted processes or use it as a
network coordination protocol.

## Dependency vulnerabilities

If a report originates in Oxigraph, Fjall, PyO3, Maturin, or another dependency,
contacting Oxiland is still appropriate when the affected behavior is reachable
through a released Oxiland artifact. Maintainers will help route and coordinate
the issue without claiming ownership of the upstream fix.

## Disclosure and fixes

When a vulnerability is confirmed, the intended process is:

1. determine affected release lines and practical mitigations;
2. coordinate privately with relevant upstream maintainers;
3. prepare fixes, regression tests, and release notes;
4. publish patched artifacts and an advisory when users can act;
5. credit the reporter unless anonymity is requested.

Public details may be limited when disclosure would create unreasonable risk
before supported users can update.
