# Fuzzing

The checked-in corpus covers malformed RDF parser bytes and C handle lifecycle
sequences. Run with nightly `cargo-fuzz`:

```text
cargo +nightly fuzz run rdf_parser -- -max_total_time=3600
cargo +nightly fuzz run c_lifecycle -- -max_total_time=3600
```

Crashes are retained under the matching corpus directory after minimization.
The 0.10 release bundle records duration, target, revision, and zero unresolved
findings; merely compiling these targets does not satisfy the release gate.
