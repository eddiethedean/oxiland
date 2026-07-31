# Security policy

## Supported versions

| Version | Supported |
|---|---|
| 0.7.x | Yes |
| 0.6.x | Yes |
| 0.5.x | Security fixes only when practical; prefer upgrading to 0.7 |
| 0.4.x | Security fixes only when practical; prefer upgrading to 0.7 |
| 0.3.x | Security fixes only when practical; prefer upgrading to 0.7 |
| 0.2.x | Security fixes only when practical; prefer upgrading to 0.7 |
| 0.1.x | Security fixes only when practical; prefer upgrading to 0.7 |
| < 0.1 | No |

Oxiland is pre-1.0. Public APIs and on-disk formats may change; security fixes
may ship as breaking 0.x releases when required.

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Email the maintainer at **odosmatthews@gmail.com** with:

- a description of the issue and impact;
- steps to reproduce or a proof of concept if available;
- affected crate version and platform.

You should receive an acknowledgement within a few business days. We will
coordinate a fix and disclosure timeline. If the issue also affects Oxigraph or
another dependency, we will help route it appropriately.

## Scope notes

- The primary `oxiland` crate forbids `unsafe` code. Memory-safety issues in
  dependencies should be reported upstream when possible.
- Fjall persistence (`Model::open`) is a local durable working store (format
  v1), not a hardened multi-tenant database; treat store paths as trusted.
- A future `oxiland-capi` crate will carry separate FFI security review.
