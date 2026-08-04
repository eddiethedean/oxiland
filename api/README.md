# Public API snapshots

This directory holds **curated** public-surface snapshots used as release and
contributor gates. They are not a substitute for rustdoc or the Python API
guide.

| File | Purpose |
|---|---|
| `oxiland-public-api.txt` | Owned Rust public symbols allowlist for the `oxiland` crate |
| `oxiland-python.pyi` | Python stub snapshot checked against the published typing surface |

## How to regenerate and check

From the repository root:

```console
scripts/generate-public-api.sh check
```

Regenerate (update the committed snapshot) only when an intentional public API
change is part of the release:

```console
scripts/generate-public-api.sh update
```

Python stub checks also use `scripts/check-python-stub-snapshot.sh` (see
`CONTRIBUTING.md`).

## How to read a diff

- **Added symbols** need changelog notes and, for compatibility work, inventory
  or parity updates when they map Redland workflows.
- **Removed or renamed symbols** are breaking in 0.x and require migration notes
  in `CHANGELOG.md`.
- Do not expand the allowlist for temporary helpers or private re-exports.

## Related docs

- Rust API: <https://docs.rs/oxiland>
- Python API: [docs/users/python-api.md](../docs/users/python-api.md)
- Contributing checks: [CONTRIBUTING.md](../CONTRIBUTING.md)
