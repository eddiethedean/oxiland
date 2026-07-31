# Examples

Copy-paste starting points. From a repository checkout unless noted.

## Rust (`examples/`)

| Example | What it shows | Run |
|---|---|---|
| `quick_start.rs` | Model + ASK | `cargo run --example quick_start` |
| `select.rs` | SELECT bindings | `cargo run --example select` |
| `construct.rs` | CONSTRUCT | `cargo run --example construct` |
| `update.rs` | SPARQL Update | `cargo run --example update` |
| `contexts.rs` | Named graphs + find | `cargo run --example contexts` |
| `parse_serialize.rs` | Turtle → N-Triples | `cargo run --example parse_serialize` |
| `progressive_load.rs` | Progressive parse load | `cargo run --example progressive_load` |
| `persistent_transaction.rs` | Fjall open + transaction | `cargo run --example persistent_transaction` |
| `std_replacements.rs` | Hash/list → std notes | `cargo run --example std_replacements` |

## Python (`python/examples/`)

Install the package first (`pip install oxiland` or `maturin develop` in
`python/`).

| Example | What it shows | Run |
|---|---|---|
| `quick_start.py` | Model + ASK | `python python/examples/quick_start.py` |
| `select.py` | SELECT | `python python/examples/select.py` |
| `parse_serialize.py` | load / serialize | `python python/examples/parse_serialize.py` |
| `persistent.py` | `Model.open` + transaction | `python python/examples/persistent.py` |

## Guides

- [Getting started](getting-started.md)
- [Python](python.md)
- [SPARQL](sparql.md)
- [Persistence](persistence.md)
- [CLI](cli.md)
