# Upgrading Oxiland

Tip **0.12.0** is the current published line. Read `CHANGELOG.md` for the
release you are leaving and the release you are entering. Export N-Quads before
any upgrade that might touch store format or major API surfaces.

## Format reopen

Persistent format v1 reopens across **0.4.x–0.12.x** patch lines. Export
N-Quads before any future format-v2 migration. Pre-0.4 experimental stores
without metadata require `Model::migrate_legacy_store` /
`Model.migrate_legacy_store` as a controlled maintenance step.

## 0.10 → 0.11

- Demonstrated Redland parity on the frozen matrix (C-oracle differentials,
  librdf-compat packaging).
- Keep reading [C ABI limitations](c-abi-limitations.md); behavioral gaps remain
  explicit.
- Pin Rust/Python packages to `0.11.0` only if you must stay on that line;
  prefer current **0.12.x** for new work.

## 0.11 → 0.12

- Competitive-parity performance gate (ADR-028) closed on the committed
  three-host bundle; host-scoped strict wins after library-path isolation are
  documented separately and are **not** a blanket faster-than-Redland claim.
- C package version is `0.12.0` (`publish = false`); rebuild from a matching
  checkout.
- Public API snapshot and Python stubs track tip 0.12.0—re-run application
  tests after bumping.

## Checklist

1. Read the changelog and [support policy](../support.md).
2. Quiesce writers; `sync()`; export N-Quads.
3. Open a staging copy with `create=false` / `create=False`.
4. Run representative ASK/SELECT and backup/restore checks.
5. Upgrade production only after staging validation.

See [Rust production](rust-production.md#upgrade-runbook) and
[Python production](python-production.md) for the full runbooks.
