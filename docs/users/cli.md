# CLI (`oxiland-cli`)

[`oxiland-cli`](https://crates.io/crates/oxiland-cli) is an rdfproc-**shaped**
command-line tool over the safe Rust facade. It is **not** a drop-in binary for
native Redland `rdfproc`.

Each invocation opens the store, runs **one** command, and exits. There is no
multi-command interactive session—chain commands against a Fjall path for
durable workflows.

## Install

```console
cargo install oxiland-cli
# or from a workspace checkout:
cargo run -p oxiland-cli -- --help
```

Requires Rust **1.87+**.

## Global options

| Flag | Meaning |
|---|---|
| `-s, --storage` | `memory` or `fjall` (default `fjall`) |
| `-n, --new` | Create the durable store directory if missing (required for new paths) |
| `-q, --quiet` | Suppress informational messages |
| `-o, --output` | Serialization syntax for print/serialize/find (default `nquads`) |
| `-r, --results` | SPARQL results format (default `xml`) |
| `-V, --version` | Print version |

Usage shape: `oxiland-cli [OPTIONS] <STORE_NAME> <COMMAND>`.

With `-s memory`, use store name `memory`. With `-s fjall`, use a filesystem
path.

## Commands

| Command | Purpose |
|---|---|
| `parse` | Parse RDF into the store (collecting / all-or-nothing load) |
| `parse-stream` | Progressive streaming load (may leave partial data on failure) |
| `serialize` | Serialize the store |
| `print` | Print the store as triples/quads |
| `add` | Add a statement |
| `remove` | Remove a statement |
| `find` | Find matching statements (`-` wildcards) |
| `query` | Run a SPARQL query |
| `contexts` | List named graph contexts |

Run `oxiland-cli help <command>` for per-command flags (syntax, base IRI, etc.).

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
| `memory` | `Model::new` (store name should be `memory`) |
| `fjall` | `Model::open` format v1 at the given path; use `-n` to create |
| other | Error |

## See also

- [Persistence](persistence.md)
- [Migration from Redland](../evaluators/migration-from-redland.md)
- Design: [0.6-cli-rdfproc.md](../design/0.6-cli-rdfproc.md)
