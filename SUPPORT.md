# Support policy

Oxiland is pre-1.0 open-source software maintained on a best-effort basis. This
policy describes which release lines receive attention, what compatibility
users can rely on, and how to request help effectively.

## Supported release lines

| Release line | Status | Expected maintenance |
|---|---|---|
| **0.7.x** | Current | Bug and security fixes as practical |
| **0.6.x** | Maintenance | Security fixes when practical; upgrade recommended |
| **0.4.x–0.5.x** | Legacy | Security fixes only when practical; upgrade strongly recommended |
| **0.1.x–0.3.x** | Legacy | Security or critical fixes only when practical; no routine maintenance |
| Earlier / unreleased snapshots | Unsupported | No compatibility or migration commitment |

Support applies to released artifacts from crates.io and PyPI, not arbitrary
commits or locally modified builds.

## Supported environments

| Surface | Current supported boundary |
|---|---|
| Rust crate | Rust 1.87+ on the CI/release target matrix |
| Python package | CPython 3.10–3.14 where a matching published wheel exists |
| CLI | Same Rust and operating-system boundary as the Rust crate |
| Persistent storage | Oxiland format v1; local trusted filesystem |

An environment is supported only when all required release artifacts exist for
its interpreter, target, and architecture. Best-effort assistance may be given
for other platforms, but it is not a release commitment.

## Stability expectations before 1.0

- Public Rust and Python APIs may change in minor 0.x releases.
- Breaking changes should appear in `CHANGELOG.md` with a migration note.
- Patch releases should remain API-compatible within their minor line unless a
  security or data-integrity issue requires otherwise.
- Format v1 stores reopen across **0.4.x–0.7.x patch lines**. A future format-v2
  change requires an explicit migration or export path.
- N-Quads is the portable backup and major-upgrade continuity format.
- Formal long-term deprecation windows and MSRV guarantees are 1.0 goals, not
  current promises.

## What support includes

Maintainers may provide:

- clarification of documented behavior;
- reproduction and triage of bugs on supported releases;
- review of minimal fixes and documentation improvements;
- security coordination through the private reporting process;
- migration guidance within the published compatibility boundary.

Support does not currently include:

- a response-time or resolution-time SLA;
- paid support, managed hosting, or operational on-call service;
- recovery of user data or administration of deployed stores;
- custom platform wheels, storage backends, or downstream integrations;
- guarantees for undocumented engine escape hatches or modified builds.

## Request help

Use [GitHub Issues](https://github.com/eddiethedean/oxiland/issues) for public
bugs and questions. Include:

- Oxiland version and installation source;
- Rust version or Python version, operating system, and architecture;
- storage backend and relevant feature flags;
- a minimal reproducible example and exact command;
- expected behavior, actual behavior, and complete error category/message;
- whether the issue reproduces with an in-memory model;
- sanitized sample RDF or SPARQL when relevant.

Do not attach production stores, credentials, private RDF, or sensitive query
content. Create a minimal sanitized fixture instead.

Consult the [documentation](https://oxiland.readthedocs.io/),
[FAQ](https://oxiland.readthedocs.io/en/latest/users/faq/), and
[changelog](https://github.com/eddiethedean/oxiland/blob/main/CHANGELOG.md)
before filing.

## Security and private reports

Do not use a public issue for a suspected vulnerability. Follow the
[security policy](https://github.com/eddiethedean/oxiland/blob/main/SECURITY.md).
Operational incidents without a suspected security impact may use the normal
issue tracker after sensitive details are removed.

## End-of-support changes

Changes to supported versions are published in this policy and should be called
out in release notes. A release line may receive an urgent security fix without
reopening routine maintenance for that line.
