# oxiland-cli

rdfproc-compatible command workflows over the [Oxiland](https://crates.io/crates/oxiland)
safe Rust facade (ADR-019). This is **not** a drop-in binary for native `rdfproc`.

```console
cargo run -p oxiland-cli -- --help
oxiland-cli memory parse ./data.ttl turtle
oxiland-cli ./store.db find - - - 
```

See `docs/users/cli.md` and `docs/design/0.6-cli-rdfproc.md`.
