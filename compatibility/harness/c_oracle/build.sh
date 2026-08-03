#!/usr/bin/env bash
# Build Redland and Oxiland C oracles for the 0.11 harness.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
OUT="$ROOT/compatibility/harness/c_oracle/bin"
mkdir -p "$OUT"

detect_prefix() {
  local name="$1"
  if [[ -d "/opt/homebrew/opt/${name}" ]]; then
    echo "/opt/homebrew/opt/${name}"
  elif [[ -d "/usr/local/opt/${name}" ]]; then
    echo "/usr/local/opt/${name}"
  else
    echo ""
  fi
}

# Prefer pkg-config on Linux/MSYS; fall back to Homebrew prefixes on macOS.
CFLAGS=(-O2 -Wall -Werror)
LDFLAGS=()
if command -v pkg-config >/dev/null 2>&1 && pkg-config --exists redland 2>/dev/null; then
  # shellcheck disable=SC2207
  CFLAGS+=($(pkg-config --cflags redland raptor2 rasqal 2>/dev/null || pkg-config --cflags redland))
  # shellcheck disable=SC2207
  LDFLAGS+=($(pkg-config --libs redland raptor2 rasqal 2>/dev/null || pkg-config --libs redland))
else
  REDLAND_PREFIX="$(detect_prefix redland)"
  RAPTOR_PREFIX="$(detect_prefix raptor)"
  RASQAL_PREFIX="$(detect_prefix rasqal)"
  for p in "$REDLAND_PREFIX" "$RAPTOR_PREFIX" "$RASQAL_PREFIX"; do
    if [[ -n "$p" ]]; then
      CFLAGS+=("-I${p}/include")
      if [[ -d "${p}/include/raptor2" ]]; then
        CFLAGS+=("-I${p}/include/raptor2")
      fi
      if [[ -d "${p}/include/rasqal" ]]; then
        CFLAGS+=("-I${p}/include/rasqal")
      fi
      LDFLAGS+=("-L${p}/lib")
    fi
  done
  # MSYS2 UCRT64 Redland layout
  for p in /ucrt64 /mingw64 /d/a/_temp/msys64/ucrt64 /c/msys64/ucrt64; do
    if [[ -d "${p}/include" ]]; then
      CFLAGS+=("-I${p}/include")
      if [[ -d "${p}/include/raptor2" ]]; then
        CFLAGS+=("-I${p}/include/raptor2")
      fi
      if [[ -d "${p}/include/rasqal" ]]; then
        CFLAGS+=("-I${p}/include/rasqal")
      fi
      LDFLAGS+=("-L${p}/lib")
    fi
  done
  LDFLAGS+=(-lrdf -lraptor2 -lrasqal)
fi

# Also probe common Debian multiarch include dirs when pkg-config is incomplete.
for inc in /usr/include/raptor2 /usr/include/rasqal; do
  if [[ -d "$inc" ]]; then
    CFLAGS+=("-I${inc}")
  fi
done

CC="${CC:-cc}"

# System Redland oracle
"$CC" "${CFLAGS[@]}" \
  "$ROOT/compatibility/harness/c_oracle/oracle.c" \
  "${LDFLAGS[@]}" \
  -o "$OUT/oracle-redland"

# Oxiland librdf-compat oracle (requires packaged compat lib)
CAPI_FEATURES="${OXILAND_CAPI_FEATURES:-}"
if [[ -n "$CAPI_FEATURES" ]]; then
  # shellcheck disable=SC2086
  cargo build -p oxiland-capi --release --locked --manifest-path "$ROOT/Cargo.toml" $CAPI_FEATURES >/dev/null
else
  cargo build -p oxiland-capi --release --locked --manifest-path "$ROOT/Cargo.toml" >/dev/null
fi
"$ROOT/scripts/package-librdf-compat.sh" >/dev/null
COMPAT="$ROOT/target/release/compat"
INC="$ROOT/crates/oxiland-capi/include"
OX_LDFLAGS=(-L"$COMPAT" -lrdf)
# ELF needs rpath; Mach-O uses -Wl,-rpath; MSVC/MinGW may ignore it.
case "$(uname -s)" in
  Darwin) OX_LDFLAGS+=(-Wl,-rpath,"$COMPAT") ;;
  Linux) OX_LDFLAGS+=(-Wl,-rpath,"$COMPAT") ;;
esac
"$CC" -O2 -Wall -Werror \
  -I"$INC" \
  "$ROOT/compatibility/harness/c_oracle/oracle.c" \
  "${OX_LDFLAGS[@]}" \
  -o "$OUT/oracle-oxiland"

# Performance benches
"$CC" "${CFLAGS[@]}" \
  "$ROOT/compatibility/harness/c_oracle/perf_bench.c" \
  "${LDFLAGS[@]}" \
  -o "$OUT/perf-redland"
"$CC" -O2 -Wall -Werror \
  -I"$INC" \
  "$ROOT/compatibility/harness/c_oracle/perf_bench.c" \
  "${OX_LDFLAGS[@]}" \
  -o "$OUT/perf-oxiland"

echo "built $OUT/oracle-redland"
echo "built $OUT/oracle-oxiland"
echo "built $OUT/perf-redland"
echo "built $OUT/perf-oxiland"
