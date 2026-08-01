# 0.10 qualification bundle

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
