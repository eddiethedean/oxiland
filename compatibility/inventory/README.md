# Compatibility inventory

The versioned manifests in this directory map Redland `librdf` symbols to
Oxiland APIs.

- [`redland-1.0.17-oxiland-0.1.json`](redland-1.0.17-oxiland-0.1.json) — curated
  0.1 core-model slice.

Validate with:

```console
python3 scripts/check-inventory.py
```

Full header-derived generation is planned with the native Redland oracle
harness. Until then, curated milestone slices are the source of truth for
claimed inventory rows.
