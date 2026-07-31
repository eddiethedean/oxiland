# oxiland-cli

`oxiland-cli` is a command-line tool for importing, inspecting, querying, and
exporting local RDF datasets. It supports persistent Fjall-backed stores and
one-shot in-memory validation workflows.

```console
cargo install oxiland-cli
oxiland-cli --help
```

## Quick start

Create a store and import Turtle:

```console
oxiland-cli --new ./catalog parse ./catalog.ttl
```

Run a SELECT query and emit CSV:

```console
oxiland-cli --results csv ./catalog query sparql - \
  'SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 20'
```

Export the complete dataset as N-Quads:

```console
oxiland-cli --output nquads ./catalog serialize > catalog.nq
```

## Command model

```text
oxiland-cli [GLOBAL OPTIONS] <STORE> <COMMAND> [COMMAND OPTIONS]
```

Each invocation opens one store, runs one command, and exits. Use a persistent
store for multi-command workflows; the in-memory backend starts empty on every
invocation.

| Command | Purpose |
|---|---|
| `parse` | Parse the complete RDF input before insertion |
| `parse-stream` | Insert progressively; valid statements before an error may remain |
| `serialize` | Write the complete dataset in an RDF syntax |
| `print` | Print the dataset using the global output syntax |
| `add` / `remove` | Mutate one simple statement |
| `find` | Match statement fields, using `-` as a wildcard |
| `query` | Execute SPARQL ASK, SELECT, CONSTRUCT, or DESCRIBE |
| `contexts` | List named graph IRIs |

Machine-readable command results are written to standard output. Status and
errors are written to standard error. `--quiet` suppresses informational status
messages but never suppresses failures. Success exits with status 0; command,
input, store, and serialization failures exit nonzero.

## Safety notes

- `--new` is required to create a missing persistent store.
- `parse` avoids partial data on RDF parse failure; `parse-stream` explicitly
  allows partial progress.
- N-Quads is the recommended portable backup format for named graphs.
- The CLI can export named-graph N-Quads but cannot import an arbitrary
  multi-graph N-Quads/TriG dataset; restore that backup through the Rust or
  Python `Model.import_nquads` API.
- The CLI is inspired by Redland `rdfproc` workflows but is not syntax- or
  binary-compatible with `rdfproc`.
- Typed and language-tagged literals are not accepted by `add`, `remove`, or
  `find`; import RDF or use SPARQL Update through an application for those
  values.

Read the complete [CLI guide](https://oxiland.readthedocs.io/en/latest/users/cli/)
for global options, term syntax, automation, formats, and troubleshooting.

## License

Apache-2.0 OR MIT
