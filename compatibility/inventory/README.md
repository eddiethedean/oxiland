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

Format name/MIME/extension dispositions:
[`../baseline/format-matrix.json`](../baseline/format-matrix.json).

Validate with:

```console
python3 scripts/check-inventory.py
```

Full header-derived generation remains planned with the broader native Redland
oracle harness. Until then, curated milestone slices are the source of truth
for claimed inventory rows.
