# oxiland-cli

rdfproc-shaped command workflows over the [Oxiland](https://crates.io/crates/oxiland)
safe Rust facade (ADR-019). This is **not** a drop-in binary for native `rdfproc`.

```console
cargo install oxiland-cli
cargo run -p oxiland-cli -- --help
oxiland-cli -s memory memory parse ./data.ttl --syntax turtle
oxiland-cli -n -s fjall ./store.db find - - -
```

See [`docs/users/cli.md`](../../docs/users/cli.md) and
[`docs/design/0.6-cli-rdfproc.md`](../../docs/design/0.6-cli-rdfproc.md).
