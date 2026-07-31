# Python API

Oxiland’s Python surface is typed with PEP 561 stubs shipped in the wheel
(`oxiland/py.typed`, `oxiland/__init__.pyi`). There is no separate Sphinx autodoc
site yet—use this page plus IDE completion.

## Entry points

| Area | Primary symbols |
|---|---|
| Terms | `NamedNode`, `BlankNode`, `Literal`, `DefaultGraph`, `Triple`, `Quad` |
| Store | `Model`, `Transaction`, `FindIter` |
| I/O | `Syntax`, `parse`, `parse_path`, `load`, `load_path`, `serialize`, `serialize_path` |
| SPARQL | `query`, `update`, `serialize_results`, `Solution`, `SolutionsIter`, `TriplesIter` |
| Utility | `DigestAlgorithm`, `digest_hex`, `digest_bytes`, `Namespace`, `vocab` |
| Errors | `OxilandError` and typed subclasses (`ParseError`, `OpenStoreError`, …) |

## Where to look

- Narrative guide: [Python package](python.md)
- Stub source in the repo: [`python/oxiland.pyi`](https://github.com/eddiethedean/oxiland/blob/main/python/oxiland.pyi)
- Runnable examples: [Examples](examples.md)
- Rust API (related semantics): [docs.rs/oxiland](https://docs.rs/oxiland)

## Import notes

```python
import oxiland
from oxiland import Model, query
import importlib

rdf = importlib.import_module("oxiland.vocab.rdf")
```

`oxiland.vocab` (and children such as `oxiland.vocab.rdf`) are registered in
`sys.modules` for submodule imports.
