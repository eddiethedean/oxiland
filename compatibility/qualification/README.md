# Qualification bundles

## 0.12 performance qualification

`0.12-matrix.json` checksums the draft suite and corrected native C benchmark
and freezes the intended Linux, macOS, and Windows release-default evidence
paths. Native 0.12 samples belong under `performance/0.12/`; they must not
overwrite the retained 0.11 diagnostic bundle.

Run `python3 scripts/check-0.12-release.py`. Until measured RSS/disk budgets
are reviewed, both the matrix and suite remain `draft`, and this command fails
closed. A release pass additionally requires exact native profile coverage,
production-build provenance, unique executions, artifact hashes, a clean
candidate revision, and refreshed green 0.11 parity evidence for that same
revision.

## 0.11 release qualification

Checked-in 0.11 evidence lives beside this README:

- `0.11-matrix.json`, `0.11-parity-evidence.json`, `0.11-soak.json`,
  `0.11-fuzz.json`, `0.11-abi-swap.json`
- `raw/` six-cell C-oracle differentials
- `performance/` native host benches (`synthetic: false`)

Run `python3 scripts/check-0.11-release.py`. Tip CI also runs
`.github/workflows/qualify-0.11.yml`. Release tags `v0.11.*` re-run the checker
from `.github/workflows/release.yml`.

## 0.10 qualification scaffold

`0.10-matrix.json` is frozen scaffold input. The 0.10 integrity check is
fail-closed until this directory also contains all of the following:

- `../inventory/redland-1.0.17-oxiland-0.10.json` with no in-scope exclusion,
  mapped-only, implemented-only, or unreviewed row and C verification for every
  public `librdf_*` symbol;
- `0.10-parity-evidence.json` covering every ID in `required_profile_ids`, with
  exact candidate symbol sets and no declared skip, mismatch, quarantine, or
  deviation;
- `performance/{target}__release-default.json` for every frozen performance
  target, retaining all raw samples and resource-budget observations; and
- `0.10-soak.json` recording a completed RC soak, zero ABI resets, and no
  release blocker;
- `0.10-fuzz.json` recording fuzz targets, planned duration, smoke results, git
  revision, and unresolved findings (empty when clean); and
- synthetic raw-sample fixtures under `performance/` for every frozen
  performance profile, labeled as synthetic in their provenance fields.

Run `python3 scripts/check-0.10-release.py`. Missing scaffold data is a failure.
The validator proves that the candidate inventory, schemas, profile coverage,
and smoke records are internally consistent. It does not prove native
cross-platform behavior, faster-than-Redland performance, C source/binary
compatibility, or release-duration fuzzing. Those claims require the raw,
revision-bound 0.11 evidence described in `docs/milestones/0.11.md`.
