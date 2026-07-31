# Compatibility inventory

The versioned manifests in this directory map Redland `librdf` symbols to
Oxiland APIs.

- [`redland-1.0.17-oxiland-0.1.json`](redland-1.0.17-oxiland-0.1.json) — curated
  0.1 core-model slice.
- [`redland-1.0.17-oxiland-0.2.json`](redland-1.0.17-oxiland-0.2.json) — curated
  0.2 RDF I/O slice.
- [`redland-1.0.17-oxiland-0.3.json`](redland-1.0.17-oxiland-0.3.json) — curated
  0.3 SPARQL query/update/results slice.
- [`redland-1.0.17-oxiland-0.4.json`](redland-1.0.17-oxiland-0.4.json) — curated
  0.4 storage/transactions slice.
- [`redland-1.0.17-oxiland-0.5.json`](redland-1.0.17-oxiland-0.5.json) — curated
  0.5 streams/utilities/logging slice.
- [`redland-1.0.17-oxiland-0.6.json`](redland-1.0.17-oxiland-0.6.json) —
  **header-derived** full public function inventory (safe-API accounting).

Format name/MIME/extension dispositions:
[`../baseline/format-matrix.json`](../baseline/format-matrix.json).

Validate with:

```console
python3 scripts/check-inventory.py
```

Regenerate the 0.6 manifest (after reviewing classification diffs):

```console
python3 scripts/generate-redland-inventory.py
```

Inputs are pinned by
[`../baseline/redland-1.0.17.sha256`](../baseline/redland-1.0.17.sha256)
(ADR-021).
