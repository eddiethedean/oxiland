# Compatibility harness

Scripts that produce machine-readable oracle and differential evidence for
Oxiland 0.2 RDF I/O.

| Script | Purpose |
|---|---|
| `oracle_smoke.py` | Run `rapper` against the Turtle smoke fixture |
| `differential_smoke.py` | Compare Oxiland and `rapper` statement counts |

Curated standards cases live under
[`../conformance/`](../conformance/) and are executed by
`cargo test --test conformance` through the public Oxiland facade.

```console
python3 compatibility/harness/oracle_smoke.py
python3 compatibility/harness/differential_smoke.py
cargo test --test conformance
```

Machine-local `*-result.json` outputs are gitignored; regenerate them locally
or in CI rather than committing host-specific paths.