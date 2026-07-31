# Support policy

Oxiland is pre-1.0 open source. Support is **best-effort**.

## What we support

| Line | Support |
|---|---|
| **0.7.x** (current) | Bug fixes and security fixes as practical |
| **0.6.x** | Security fixes when practical; prefer upgrading to 0.7 |
| Older 0.x | Prefer upgrading; fixes only when practical |

See also the [security policy](https://github.com/eddiethedean/oxiland/blob/main/SECURITY.md)
for vulnerability reporting and the security-supported version table.

## Stability expectations (0.x)

- Public Rust and Python APIs may change in minor 0.x releases.
- Breaking changes should be called out in
  [CHANGELOG.md](https://github.com/eddiethedean/oxiland/blob/main/CHANGELOG.md).
- On-disk **format v1** reopen is promised for patch releases in **0.4.x–0.7.x**
  (see the
  [persistence guide](https://github.com/eddiethedean/oxiland/blob/main/docs/users/persistence.md)).
  A future format v2 would ship with an explicit migrator.
- Deprecation: when a public API is removed, prefer a changelog note and a
  migration hint in the release that removes it. Formal long-term deprecation
  windows are a **1.0** charter goal, not a current 0.x guarantee.

## How to get help

- Bugs and questions: [GitHub Issues](https://github.com/eddiethedean/oxiland/issues)
- Security: private email per
  [SECURITY.md](https://github.com/eddiethedean/oxiland/blob/main/SECURITY.md)
- Docs and guides: [oxiland.readthedocs.io](https://oxiland.readthedocs.io/)

There is no paid support contract or SLA today.
