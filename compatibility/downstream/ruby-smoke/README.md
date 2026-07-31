# Ruby `redland` binding smoke

Full gem linkage against Oxiland is **not** a hard 0.9 gate (deviation D-09-01).
When Ruby and native build tools are available, maintainers may attempt:

```console
gem install redland -- --with-opt-dir=/path/to/oxiland/prefix
```

Expected outcome on tip: link failures for iostream/iterator/print symbols that
remain `excluded` in the 0.9 inventory. Document any new findings in
`compatibility/downstream/README.md`.
