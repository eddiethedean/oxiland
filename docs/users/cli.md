# Oxiland command line

`oxiland-cli` manages local RDF datasets from scripts and interactive shells.
It can create or open a persistent store, import RDF, inspect statements and
graphs, execute SPARQL queries, and export standards RDF.

## Install

`oxiland-cli` requires Rust 1.87 or newer:

```console
cargo install oxiland-cli
oxiland-cli --version
```

From a repository checkout:

```console
cargo run -p oxiland-cli -- --help
```

## Invocation model

```text
oxiland-cli [GLOBAL OPTIONS] <STORE_NAME> <COMMAND> [COMMAND OPTIONS]
```

Each process opens a store, runs exactly one command, and exits. There is no
interactive multi-command session. Use the same persistent path across
invocations to build a workflow.

With `--storage fjall`, `STORE_NAME` is a filesystem path. With
`--storage memory`, use `memory`; that model starts empty and is discarded when
the command exits.

## Global options

| Option | Default | Meaning |
|---|---|---|
| `-s, --storage <TYPE>` | `fjall` | `fjall` or `memory` (`mem` is accepted) |
| `-n, --new` | off | Allow creation of a missing persistent store |
| `-q, --quiet` | off | Suppress informational status on stderr |
| `-o, --output <SYNTAX>` | `nquads` | RDF syntax for `print`, `find`, graph queries, and default `serialize` |
| `-r, --results <FORMAT>` | `xml` | ASK/SELECT result format: XML, JSON, CSV, or TSV |
| `-V, --version` | — | Print the package version |

Data output goes to stdout. Status and errors go to stderr, which makes shell
redirection safe:

```console
oxiland-cli --quiet --output nquads ./catalog serialize > catalog.nq
```

Successful commands exit 0. Argument, RDF, SPARQL, storage, and I/O failures
exit nonzero and include an `oxiland-cli:` diagnostic on stderr.

## Create and import a dataset

```console
oxiland-cli --new ./catalog parse ./data.ttl
```

Syntax is inferred from an unambiguous path extension. Override it for files
without a useful extension:

```console
oxiland-cli --new ./catalog parse ./upload.data \
  --syntax turtle \
  --base https://example.com/
```

Choose import failure semantics deliberately:

| Command | Behavior |
|---|---|
| `parse` | Parses the complete input before insertion; malformed RDF leaves the model unchanged |
| `parse-stream` | Inserts progressively; statements before a later parse error may remain durable |

Use `parse` for normal imports. Use `parse-stream` only when partial progress is
acceptable and recovery is defined.

Supported RDF names are `turtle`, `ntriples`, `nquads`, `trig`, and `rdfxml`.
Unknown or ambiguous formats fail instead of being guessed.

## Inspect and export

Print all quads as N-Quads:

```console
oxiland-cli --output nquads ./catalog print
```

Serialize as Turtle when the dataset is graph-only:

```console
oxiland-cli ./catalog serialize --syntax turtle > catalog.ttl
```

Use N-Quads or TriG for a dataset containing named graphs. Turtle, N-Triples,
and RDF/XML cannot represent multiple graph names.

List named graph IRIs:

```console
oxiland-cli ./catalog contexts
```

## Add and remove statements

Arguments use plain absolute IRIs, `_:` blank-node identifiers, and simple
literals. Quote literals at the shell boundary when they contain whitespace:

```console
oxiland-cli ./catalog add \
  https://example.com/alice \
  https://schema.org/name \
  'Alice Smith'

oxiland-cli ./catalog remove \
  https://example.com/alice \
  https://schema.org/name \
  'Alice Smith'
```

Add a final graph IRI to operate in a named graph:

```console
oxiland-cli ./catalog add \
  https://example.com/alice \
  https://schema.org/name \
  Alice \
  https://example.com/graph/people
```

The node-argument grammar intentionally does not support typed or
language-tagged literals. Import RDF for those values or use an application
SPARQL Update API.

## Find statements

`find` accepts subject, predicate, object, and an optional context. Use `-` as a
wildcard:

```console
# Everything about Alice, across graphs
oxiland-cli ./catalog find https://example.com/alice - -

# Everything in one named graph
oxiland-cli ./catalog find - - - https://example.com/graph/people
```

`find` output uses the global `--output` syntax. The implementation currently
collects matching quads before serialization, so bound patterns are preferable
for large stores.

## Query with SPARQL

The query command keeps an rdfproc-shaped language and URI position:

```text
oxiland-cli [OPTIONS] <STORE> query <LANGUAGE> <URI-OR-DASH> <SPARQL>
```

Use `sparql` or `-` for the language and `-` for the unused query URI:

```console
oxiland-cli --results csv ./catalog query sparql - \
  'SELECT ?s ?name WHERE { ?s <https://schema.org/name> ?name } LIMIT 100'
```

ASK and SELECT use `--results` (`xml`, `json`, `csv`, or `tsv`). CONSTRUCT and
DESCRIBE use the RDF `--output` syntax.

RDQL and SPARQL Update are not CLI commands. Use the Rust or Python API for
updates.

## In-memory mode

```console
oxiland-cli --storage memory memory parse ./data.ttl
```

Because every invocation performs one command and exits, in-memory mode is
useful for input validation and one-shot empty-dataset checks—not for a sequence
of import, query, and export commands. Use a persistent path for those workflows.

## Backup workflow

Export named graphs to a file outside the live store directory:

```console
oxiland-cli --quiet --output nquads ./catalog serialize \
  > ./backups/catalog.nq
```

Verify restoration into a new path:

```console
oxiland-cli --new ./restore-check parse ./backups/default-graph.nt --syntax ntriples
oxiland-cli ./restore-check query sparql - 'ASK { ?s ?p ?o }'
```

Import merges statements into the target dataset. Restore into a new store when
replacement semantics are required.

!!! warning "Named-graph restore"

    The CLI can export a complete named-graph dataset as N-Quads, but its
    parser does not expose the dataset graph target needed to import arbitrary
    named graphs from one N-Quads or TriG file. Restore those backups with Rust
    `Model::import_nquads_from_path` or Python `Model.import_nquads`. CLI restore
    works directly for graph-only N-Triples/Turtle/RDF/XML data.

## Scope and migration

The CLI is inspired by Redland `rdfproc` workflows, but it is not a binary,
flag, output, or session-level drop-in. It supports the documented Oxiland
storage and syntax matrix and rejects unknown backends and query languages.

See [Migration from Redland](../evaluators/migration-from-redland.md),
[Persistence](persistence.md), and the [FAQ](faq.md).
