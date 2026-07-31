# CLI (`oxiland-cli`)

Oxiland 0.6 ships [`oxiland-cli`](https://crates.io/crates/oxiland-cli), an
rdfproc-shaped command-line tool over the safe Rust facade (ADR-019). It is
**not** a drop-in binary for native Redland `rdfproc`.

Each invocation opens the store, runs **one** command, and exits. There is no
multi-command interactive session—chain commands against a Fjall path for
durable workflows.

## Install

```console
cargo install oxiland-cli
# or from a workspace checkout:
cargo run -p oxiland-cli -- --help
```

## Quick examples

```console
# Parse Turtle into a new Fjall store (-n required when the path does not exist)
oxiland-cli -n -s fjall ./mystore parse ./data.ttl --syntax turtle

# Find all quads (default -o nquads supports named graphs)
oxiland-cli -s fjall ./mystore find - - -

# SPARQL SELECT (CSV results)
oxiland-cli -s fjall -r csv ./mystore query - - \
  'SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10'

# In-memory one-shot (cannot persist across process boundaries)
oxiland-cli -s memory memory parse ./data.ttl --syntax turtle
```

## Storage

| `-s` value | Behavior |
|---|---|
| `memory` | `Model::new` (store-name should be `memory`) |
| `fjall` | `Model::open` format v1 at the given path; use `-n` to create |
| other | Error |

## See also

- Design: [0.6-cli-rdfproc.md](../design/0.6-cli-rdfproc.md)
- Migration: [migration-from-redland.md](../evaluators/migration-from-redland.md)
