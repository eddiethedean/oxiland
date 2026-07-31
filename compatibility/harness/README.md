# Compatibility harness

Scripts that produce machine-readable oracle, differential, and facade-smoke
evidence for Oxiland RDF I/O (0.2) and SPARQL (0.3).

| Script | Purpose |
|---|---|
| `oracle_smoke.py` | Run `rapper` against the Turtle smoke fixture |
| `differential_smoke.py` | Compare Oxiland and `rapper` statement counts |
| `sparql_smoke.py` | Oxiland SPARQL facade smoke (`oxiland-facade` classification) |

Curated standards cases live under
[`../conformance/`](../conformance/) and are executed by
`cargo test --test conformance` through the public Oxiland facade.

```console
python3 compatibility/harness/oracle_smoke.py
python3 compatibility/harness/differential_smoke.py
python3 compatibility/harness/sparql_smoke.py
cargo test --test conformance
```

Machine-local `*-result.json` outputs are gitignored; regenerate them locally
or in CI rather than committing host-specific paths.
