# Redland / Raptor baseline for Oxiland 0.2

Pinned compatibility inputs for the RDF I/O milestone.

| Component | Version | Notes |
|---|---|---|
| Redland `librdf` API | 1.0.17 | Manual labeled 1.0.18 |
| Raptor | 2.0.16 (typical packaged) | Oracle via `rapper` CLI |
| Rasqal | deferred | 0.3 ships an Oxiland facade SPARQL smoke harness; native Rasqal differential oracles remain future work |
| Oxigraph | 0.5.9 | Exact pin in `Cargo.toml` |
| oxrdfio | 0.2.5 | Re-exported by Oxigraph |

## Checksums and acquisition

Canonical upstream sources (record SHA-256 when vendoring tarballs):

- Redland: `https://download.librdf.org/source/redland-1.0.17.tar.gz`
  (HTTP also works); SHA-256: [`redland-1.0.17.sha256`](redland-1.0.17.sha256)
- Raptor2: `http://download.librdf.org/source/raptor2-2.0.16.tar.gz`

Header-derived inventory generation:

```console
python3 scripts/generate-redland-inventory.py
```

This repository does not vendor the full native trees. CI and local oracle
scripts prefer the system `rapper` binary from `raptor2-utils` / `libraptor2`
packages and record `rapper -v` output in result metadata.

## Reproducible oracle smoke

```console
python3 compatibility/harness/oracle_smoke.py
```

When `rapper` is missing, the smoke emits a skipped result with reason rather
than a false pass.

## Format disposition summary

See [`format-matrix.json`](format-matrix.json) for Redland name / MIME /
extension dispositions used by Oxiland `Syntax` lookup.
