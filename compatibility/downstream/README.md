# Downstream C consumer matrix (0.11)

Status: active evidence package
Milestone: 0.11

## Matrix

| Consumer | Platform gate | Evidence |
|---|---|---|
| First-party expanded workflows | Linux/macOS/Windows via `oxiland-capi` CI | `crates/oxiland-capi/tests/ffi_lifecycle.rs`, ASan (Linux) |
| Frozen C source corpus (WP-11-05) | Linux/macOS with system Redland | `compatibility/downstream/corpus/` |
| Binary ABI interchange (WP-11-06) | Linux/macOS with system Redland | `compatibility/downstream/abi/` |
| Differential harness | Linux tip CI | `compatibility/downstream/differential/` |
| Selected Redland-shaped C examples | Linux tip CI | `compatibility/downstream/examples/` |
| Ruby `redland` binding smoke | Linux tip CI (soft) | `compatibility/downstream/ruby-smoke/`; deviations recorded below |

## ABI claims

- **Source compatibility:** frozen corpus compiles against system Redland and `crates/oxiland-capi/include/librdf.h` (`run-corpus.sh`).
- **Oxiland ABI:** layout/calling/lifecycle tested via FFI lifecycle + packaging smoke.
- **Redland binary `.so` swap:** exercised by `abi/run-abi-swap.sh` after `scripts/package-librdf-compat.sh` (evidence under `abi/abi-swap-result.json`).

## Known deviations

| ID | Consumer | Deviation | Disposition |
|---|---|---|---|
| D-09-01 | Ruby redland gem | Full gem link may require symbols outside the 0.9 allowlist (iostreams, iterators, print helpers) | Accepted: exercise C-layer smoke only; document gap for 0.10 |
| D-09-02 | Redland FILE*/iostream examples | Excluded from allowlist | Accepted: use string/file-path APIs |

## How to run

```console
cargo build -p oxiland-capi --locked
scripts/check-capi-symbols.sh debug
compatibility/downstream/run-smokes.sh
compatibility/downstream/corpus/run-corpus.sh
compatibility/downstream/abi/run-abi-swap.sh
```
