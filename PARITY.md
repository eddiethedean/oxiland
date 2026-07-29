# Redland parity ledger

Target: the documented Redland `librdf` 1.0.17 API (manual labeled 1.0.18).

Planned sequencing and completion rules are documented in the
[0.x roadmap](docs/ROADMAP.md) and
[compatibility plan](docs/COMPATIBILITY.md).

| Redland subsystem | Rust API | C ABI | Status |
|---|---:|---:|---|
| World / lifecycle | partial | no | RAII lifecycle and feature registry |
| URI | via Oxigraph | no | validated named-node IRIs |
| Nodes | via Oxigraph | no | URI, blank and literal terms |
| Statements | via Oxigraph | no | triples and partial matching |
| Model | partial | no | CRUD, size, contains, patterns, contexts |
| Storage | partial | no | memory; RocksDB behind `rocksdb` |
| Streams / iterators | partial | no | eager safe matching; streaming API pending |
| Parser | primitive re-export | no | Oxigraph parser exposed; facade pending |
| Serializer | primitive re-export | no | Oxigraph serializer exposed; facade pending |
| SPARQL query/results | partial | no | query execution; limits/offset facade pending |
| Query update | pending | no | evaluator supports update; facade pending |
| Digests | pending | no | |
| Hashes / lists | pending | no | likely standard-library adapters |
| Heuristics / files / Unicode | pending | no | |
| Logging | pending | no | tracing adapter planned |
| Storage plug-ins | pending | no | capability mapping required |
| `rdfproc` utility | pending | n/a | |

“100% parity” is reached only when every public Redland function is represented
in a generated symbol inventory, has a documented mapping or intentional
safe-Rust replacement, and passes differential tests against native Redland.
