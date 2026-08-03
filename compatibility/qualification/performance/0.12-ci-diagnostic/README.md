# 0.12 CI diagnostic performance samples

Diagnostic uploads from GitHub Actions
[0.12 Qualification run 30848514245](https://github.com/eddiethedean/oxiland/actions/runs/30848514245)
on tip `57f3d3ddaffd74b022767d928b52e24988a53a5a`.

These files are **not** the release-gate bundle. Qualification binds to
`compatibility/qualification/performance/0.12/` via `check-0.12-release.py`.

| Host | File | Notes |
|---|---|---|
| Linux x86_64 | `x86_64-unknown-linux-gnu__release-default.json` | Fresh tip measurement (`git_revision` `57f3d3d…`) |
| macOS arm64 | `aarch64-apple-darwin__release-default.json` | Fresh tip measurement; `P-CALL-100K` missed ADR-028 median on this runner |
| Windows x86_64 | _(not duplicated)_ | CI re-uploaded the committed historical `0.12/` bundle (`34c0976…`); see that path |

Published ratio tables: `docs/reports/0.12.md`.
