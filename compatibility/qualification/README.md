# 0.10 qualification bundle

`0.10-matrix.json` is frozen input. A release tag is fail-closed until this
directory also contains all of the following:

- `../inventory/redland-1.0.17-oxiland-0.10.json` with no in-scope exclusion,
  mapped-only, implemented-only, or unreviewed row and C verification for every
  public `librdf_*` symbol;
- `0.10-parity-evidence.json` covering every ID in `required_profile_ids`, with
  exact verified-symbol sets and no skip, mismatch, quarantine, or deviation;
- `performance/{target}__release-default.json` for every frozen performance
  target, retaining all raw samples and resource-budget observations; and
- `0.10-soak.json` recording a completed RC soak, zero ABI resets, and no
  release blocker;
- `0.10-fuzz.json` recording fuzz targets, release-required duration, git
  revision, and unresolved findings (empty when clean); and
- performance raw samples under `performance/` for every frozen performance
  profile.

Run `python3 scripts/check-0.10-release.py`. Missing data is a failure. The
validator consumes raw evidence; checked-in summary prose is never accepted as
a substitute. Qualification artifacts should be produced from clean release
builds, retain the tested Git revision and host/toolchain metadata, and receive
the same provenance/signature treatment as release artifacts.
