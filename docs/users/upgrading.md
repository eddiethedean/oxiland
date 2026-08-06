# Upgrading Oxiland

Tip **0.13.0** is the current published line. Read `CHANGELOG.md` for the
release you are leaving and the release you are entering. Export N-Quads before
any upgrade that might touch store format or major API surfaces.

## Format reopen

Persistent format v1 reopens across **0.4.x–0.13.x** patch lines. Export
N-Quads before any future format-v2 migration. Pre-0.4 experimental stores
without metadata require `Model::migrate_legacy_store` /
`Model.migrate_legacy_store` as a controlled maintenance step.

## 0.10 → 0.11

- Demonstrated Redland parity on the frozen matrix (C-oracle differentials,
  librdf-compat packaging).
- Keep reading [C ABI limitations](c-abi-limitations.md); behavioral gaps remain
  explicit.
- Pin Rust/Python packages to `0.11.0` only if you must stay on that line;
  prefer current **0.13.x** for new work.

## 0.11 → 0.12

- Competitive-parity performance gate (ADR-028) closed on the committed
  three-host bundle; host-scoped strict wins after library-path isolation are
  documented separately.
- C package version tracks the tip checkout (`publish = false`); rebuild from a
  matching tag.
- Public API snapshot and Python stubs track the tip line—re-run application
  tests after bumping.

## 0.12 → 0.13

- Suite-wide faster-than-Redland gate (ADR-029) closed: Linux, macOS, and
  Windows each have three independent corrected-runner wins under the frozen
  strict suite ([0.13 report](../reports/0.13.md)).
- C SELECT/CONSTRUCT hot paths and calibrated store-cursor fast paths land in
  this line; rebuild `oxiland-capi` from `v0.13.0` (or tip) for C consumers.
- Pin installs to `0.13.0` / `oxiland==0.13.0`; format-v1 reopen extends through
  **0.4.x–0.13.x**.

## Checklist

1. Read the changelog and [support policy](../support.md).
2. Quiesce writers; `sync()`; export N-Quads.
3. Open a staging copy with `create=false` / `create=False`.
4. Run representative ASK/SELECT and backup/restore checks.
5. Upgrade production only after staging validation.

See also [limitations](limitations.md) and [persistence](persistence.md).
