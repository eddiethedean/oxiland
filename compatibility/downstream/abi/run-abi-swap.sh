#!/usr/bin/env bash
# WP-11-06: link a C program against system Redland, then load Oxiland's
# Redland-compatible shared library without rebuilding.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"

ABI_DIR="$ROOT/compatibility/downstream/abi"
OUT_DIR="${ABI_OUT_DIR:-$ROOT/target/abi-swap}"
COMPAT_DIR="$ROOT/target/release/compat"
RESULT_JSON="$ABI_DIR/abi-swap-result.json"
mkdir -p "$OUT_DIR"

if ! command -v pkg-config >/dev/null 2>&1 || ! pkg-config --exists redland; then
  echo "error: system Redland (pkg-config redland) is required" >&2
  exit 1
fi

REDLAND_CFLAGS="$(pkg-config --cflags redland)"
REDLAND_LIBS="$(pkg-config --libs redland)"
REDLAND_LIBDIR="$(pkg-config --variable=libdir redland)"

echo "==> Building oxiland-capi release + compat packaging"
cargo build -p oxiland-capi --release --locked
"$ROOT/scripts/package-librdf-compat.sh" release

OS="$(uname -s)"
case "$OS" in
  Darwin)
    COMPAT_LIB="$COMPAT_DIR/librdf.0.dylib"
    LOAD_NAME="librdf.0.dylib"
    ;;
  Linux)
    COMPAT_LIB="$COMPAT_DIR/librdf.so.0"
    LOAD_NAME="librdf.so.0"
    ;;
  *)
    echo "error: unsupported OS for ABI swap: $OS" >&2
    exit 1
    ;;
esac

if [[ ! -f "$COMPAT_LIB" ]]; then
  echo "error: missing compat library $COMPAT_LIB" >&2
  exit 1
fi

PROBE="$OUT_DIR/abi_probe"
echo "==> Compiling abi_probe against system Redland"
# shellcheck disable=SC2086
cc -std=c11 -Wall -Wextra -Werror $REDLAND_CFLAGS \
  "$ABI_DIR/abi_probe.c" $REDLAND_LIBS \
  -o "$PROBE"

# On macOS Homebrew, LC_LOAD_DYLIB is often an absolute path; rewrite to a bare
# soname so DYLD_LIBRARY_PATH can select Oxiland without a source rebuild.
if [[ "$OS" == Darwin ]]; then
  OLD_ID="$(otool -L "$PROBE" | awk '/librdf/ {print $1; exit}')"
  if [[ -n "${OLD_ID:-}" && "$OLD_ID" == /* ]]; then
    echo "  install_name_tool: $OLD_ID -> $LOAD_NAME (loader-path only; no rebuild)"
    install_name_tool -change "$OLD_ID" "$LOAD_NAME" "$PROBE"
  fi
fi

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    sha256sum "$1" | awk '{print $1}'
  fi
}

TARGET="$(rustc -vV | awk '/^host:/ {print $2}')"
GIT_REV="$(git rev-parse HEAD 2>/dev/null || echo unknown)"
CLEAN_PY=False
if git diff --quiet && git diff --cached --quiet; then
  CLEAN_PY=True
fi
TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
START_MS="$(python3 -c 'import time; print(int(time.time()*1000))')"

echo "==> Baseline run with system Redland on library path"
set +e
case "$OS" in
  Darwin)
    DYLD_LIBRARY_PATH="$REDLAND_LIBDIR${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}" \
      "$PROBE"
    BASELINE_RC=$?
    ;;
  Linux)
    LD_LIBRARY_PATH="$REDLAND_LIBDIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" \
      "$PROBE"
    BASELINE_RC=$?
    ;;
esac
set -e

echo "==> ABI swap run with Oxiland compat librdf"
set +e
case "$OS" in
  Darwin)
    DYLD_LIBRARY_PATH="$COMPAT_DIR${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}" \
      "$PROBE"
    SWAP_RC=$?
    ;;
  Linux)
    LD_LIBRARY_PATH="$COMPAT_DIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" \
      "$PROBE"
    SWAP_RC=$?
    ;;
esac
set -e

END_MS="$(python3 -c 'import time; print(int(time.time()*1000))')"
ELAPSED=$((END_MS - START_MS))
PASSED=0
if [[ "$BASELINE_RC" -eq 0 && "$SWAP_RC" -eq 0 ]]; then
  PASSED=1
fi

BASELINE_OK_PY=False
[[ "$BASELINE_RC" -eq 0 ]] && BASELINE_OK_PY=True
SWAP_OK_PY=False
[[ "$SWAP_RC" -eq 0 ]] && SWAP_OK_PY=True
PASSED_PY=False
[[ "$PASSED" -eq 1 ]] && PASSED_PY=True
LIB_PATH_ENV="LD_LIBRARY_PATH"
[[ "$OS" == Darwin ]] && LIB_PATH_ENV="DYLD_LIBRARY_PATH"
COMPAT_SHA="$(sha256_file "$COMPAT_LIB")"

python3 - "$RESULT_JSON" <<PY
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = {
    "schema_version": 1,
    "milestone": "0.11",
    "work_package": "WP-11-06",
    "test": "abi-swap",
    "target": "$TARGET",
    "git_revision": "$GIT_REV",
    "clean_worktree": $CLEAN_PY,
    "timestamp": "$TIMESTAMP",
    "elapsed_ms": $ELAPSED,
    "synthetic": False,
    "artifacts": {
        "probe": "$PROBE",
        "system_redland_libdir": "$REDLAND_LIBDIR",
        "oxiland_compat_library": "$COMPAT_LIB",
        "oxiland_compat_library_sha256": "$COMPAT_SHA",
    },
    "baseline": {
        "engine": "system-redland",
        "ok": $BASELINE_OK_PY,
        "exit_code": $BASELINE_RC,
    },
    "oxiland_swap": {
        "engine": "oxiland-librdf-compat",
        "ok": $SWAP_OK_PY,
        "exit_code": $SWAP_RC,
        "library_path_env": "$LIB_PATH_ENV",
        "library_path": "$COMPAT_DIR",
    },
    "comparison": {
        "passed": $PASSED_PY,
        "note": "Binary linked against system Redland headers/libs; Oxiland loaded at runtime via library path (macOS may rewrite absolute LC_LOAD_DYLIB to bare soname).",
    },
}
path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(f"wrote {path}")
PY

if [[ "$PASSED" -ne 1 ]]; then
  echo "error: ABI swap failed (baseline=$BASELINE_RC swap=$SWAP_RC)" >&2
  exit 1
fi

echo "abi-swap: passed (system Redland link + Oxiland load-without-rebuild)"
