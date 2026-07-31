# Examples

Runnable starting points for Python, Rust, C, and the command-line package. The
repository examples are executed in CI; production configuration and failure
handling are intentionally expanded in the linked guides.

## Python (`python/examples/`)

Install the published package:

```console
python -m venv .venv
source .venv/bin/activate
python -m pip install oxiland
```

From a repository checkout, run the examples at the repository root:

| Example | What it demonstrates | Run |
|---|---|---|
| `quick_start.py` | Build a model and run ASK | `python python/examples/quick_start.py` |
| `select.py` | Stream SELECT solution rows | `python python/examples/select.py` |
| `construct.py` | CONSTRUCT into a graph iterator | `python python/examples/construct.py` |
| `update.py` | SPARQL Update INSERT/DELETE | `python python/examples/update.py` |
| `parse_serialize.py` | Load Turtle and serialize N-Triples | `python python/examples/parse_serialize.py` |
| `persistent.py` | Open a durable model and commit a transaction | `python python/examples/persistent.py` |

Named-graph workflows (programmatic CRUD, N-Quads/TriG, and the current dataset
import limitation) are covered in [RDF I/O and SPARQL](python-data.md) and the
[Python API reference](python-api.md); there is not yet a dedicated
`python/examples/` script for named graphs.

The same workflows are explained in [Models and RDF terms](python-models.md),
[RDF I/O and SPARQL](python-data.md), and
[Production operations](python-production.md).

## Rust (`examples/`)

| Example | What it demonstrates | Run |
|---|---|---|
| `quick_start.rs` | Model + ASK | `cargo run --example quick_start` |
| `select.rs` | SELECT bindings | `cargo run --example select` |
| `construct.rs` | CONSTRUCT | `cargo run --example construct` |
| `update.rs` | SPARQL Update | `cargo run --example update` |
| `contexts.rs` | Named graphs + find | `cargo run --example contexts` |
| `parse_serialize.rs` | Turtle to N-Triples | `cargo run --example parse_serialize` |
| `progressive_load.rs` | Progressive parse load | `cargo run --example progressive_load` |
| `persistent_transaction.rs` | Persistent open + transaction | `cargo run --example persistent_transaction` |
| `std_replacements.rs` | Hash/list replacement notes | `cargo run --example std_replacements` |

## C (`crates/oxiland-capi/examples/`)

| Example | What it demonstrates | Run |
|---|---|---|
| `preview_workflow.c` | World → storage → model → Turtle → ASK/SELECT | see below |

From the repository root (after `cargo build -p oxiland-capi`):

```console
cc -I crates/oxiland-capi/include -L target/debug \
  crates/oxiland-capi/examples/preview_workflow.c \
  -loxiland_capi -Wl,-rpath,$PWD/target/debug \
  -o preview_workflow
./preview_workflow
```

Full install, pkg-config, and allowlist details:
[C ABI preview](c-abi.md).

## Command line

```console
# Import into a new persistent store
oxiland-cli --new ./catalog parse ./data.ttl

# Query as CSV
oxiland-cli --results csv ./catalog query sparql - \
  'SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 20'

# Export named graphs safely
oxiland-cli --quiet --output nquads ./catalog serialize > catalog.nq
```

See the [CLI guide](cli.md) for term arguments, stdout/stderr behavior, import
failure semantics, and the named-graph restore limitation.

## Next guides

- [Python overview](python.md)
- [Python API reference](python-api.md)
- [Rust getting started](getting-started.md)
- [Rust SPARQL](sparql.md)
- [Rust persistence](persistence.md)
- [CLI](cli.md)
- [C ABI preview](c-abi.md)
