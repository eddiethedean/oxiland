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

REDLAND_PREFIX="$(detect_prefix redland)"
RAPTOR_PREFIX="$(detect_prefix raptor)"
RASQAL_PREFIX="$(detect_prefix rasqal)"
CFLAGS=(-O2 -Wall -Werror)
LDFLAGS=()
for p in "$REDLAND_PREFIX" "$RAPTOR_PREFIX" "$RASQAL_PREFIX"; do
  if [[ -n "$p" ]]; then
    CFLAGS+=("-I${p}/include")
    # Homebrew nests raptor/rasqal headers under versioned subdirs.
    if [[ -d "${p}/include/raptor2" ]]; then
      CFLAGS+=("-I${p}/include/raptor2")
    fi
    if [[ -d "${p}/include/rasqal" ]]; then
      CFLAGS+=("-I${p}/include/rasqal")
    fi
    LDFLAGS+=("-L${p}/lib")
  fi
done

# System Redland oracle
cc "${CFLAGS[@]}" "${LDFLAGS[@]}" \
  "$ROOT/compatibility/harness/c_oracle/oracle.c" \
  -lrdf -lraptor2 -lrasqal \
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
cc -O2 -Wall -Werror \
  -I"$INC" -L"$COMPAT" -Wl,-rpath,"$COMPAT" \
  "$ROOT/compatibility/harness/c_oracle/oracle.c" \
  -lrdf \
  -o "$OUT/oracle-oxiland"

# Performance benches (subset of suite cases measurable via librdf C API)
cc "${CFLAGS[@]}" "${LDFLAGS[@]}" \
  "$ROOT/compatibility/harness/c_oracle/perf_bench.c" \
  -lrdf -lraptor2 -lrasqal \
  -o "$OUT/perf-redland"
cc -O2 -Wall -Werror \
  -I"$INC" -L"$COMPAT" -Wl,-rpath,"$COMPAT" \
  "$ROOT/compatibility/harness/c_oracle/perf_bench.c" \
  -lrdf \
  -o "$OUT/perf-oxiland"

echo "built $OUT/oracle-redland"
echo "built $OUT/oracle-oxiland"
echo "built $OUT/perf-redland"
echo "built $OUT/perf-oxiland"
