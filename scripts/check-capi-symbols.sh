#!/usr/bin/env bash
# Verify the library exports and public header match the frozen symbol snapshot.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ALLOWLIST="$ROOT/crates/oxiland-capi/symbols.version"
HEADER="$ROOT/crates/oxiland-capi/include/librdf.h"
PROFILE="${1:-debug}"
LIBDIR="$ROOT/target/$PROFILE"

if [[ ! -f "$ALLOWLIST" ]]; then
  echo "error: missing $ALLOWLIST" >&2
  exit 1
fi

EXPECTED_FILE="$(mktemp)"
ACTUAL_FILE="$(mktemp)"
NM_RAW="$(mktemp)"
trap 'rm -f "$EXPECTED_FILE" "$ACTUAL_FILE" "$NM_RAW"' EXIT

awk '/^[[:space:]]*librdf_/ { gsub(/;/, "", $1); print $1 }' "$ALLOWLIST" | sort -u >"$EXPECTED_FILE"

if [[ ! -s "$EXPECTED_FILE" ]]; then
  echo "error: allowlist is empty" >&2
  exit 1
fi

python3 - "$HEADER" "$EXPECTED_FILE" <<'PY'
from pathlib import Path
import re
import sys

header = Path(sys.argv[1])
expected = set(Path(sys.argv[2]).read_text(encoding="utf-8").splitlines())
declared = set(re.findall(r"\b(librdf_[A-Za-z0-9_]+)\s*\(", header.read_text(encoding="utf-8")))
if declared != expected:
    for symbol in sorted(expected - declared):
        print(f"missing header declaration: {symbol}", file=sys.stderr)
    for symbol in sorted(declared - expected):
        print(f"header declaration absent from snapshot: {symbol}", file=sys.stderr)
    raise SystemExit(1)
print(f"C header matches snapshot ({len(declared)} symbols)")
PY

LIB=""
for candidate in \
  "$LIBDIR/liboxiland_capi.so" \
  "$LIBDIR/liboxiland_capi.dylib" \
  "$LIBDIR/oxiland_capi.dll"
do
  if [[ -f "$candidate" ]]; then
    LIB="$candidate"
    break
  fi
done

if [[ -z "$LIB" ]]; then
  echo "error: built library not found under $LIBDIR (run cargo build -p oxiland-capi first)" >&2
  exit 1
fi

if ! command -v nm >/dev/null 2>&1; then
  echo "error: nm is required" >&2
  exit 1
fi

# Dynamically exported symbols (macOS: nm -gU; ELF: nm -D).
if nm -gU "$LIB" >"$NM_RAW" 2>/dev/null; then
  :
else
  nm -D "$LIB" >"$NM_RAW"
fi

# Normalize: keep librdf_* names; strip leading '_' used on macOS.
awk '{
  for (i = 1; i <= NF; i++) {
    sym = $i
    if (sym ~ /^_?librdf_/) {
      sub(/^_/, "", sym)
      print sym
    }
  }
}' "$NM_RAW" | sort -u >"$ACTUAL_FILE"

missing=0
extra=0
while IFS= read -r sym; do
  if ! grep -qxF "$sym" "$ACTUAL_FILE"; then
    echo "missing export: $sym" >&2
    missing=1
  fi
done <"$EXPECTED_FILE"

while IFS= read -r sym; do
  if [[ -z "$sym" ]]; then
    continue
  fi
  if ! grep -qxF "$sym" "$EXPECTED_FILE"; then
    echo "unexpected export: $sym" >&2
    extra=1
  fi
done <"$ACTUAL_FILE"

if [[ "$missing" -ne 0 || "$extra" -ne 0 ]]; then
  echo "oxiland-capi symbol allowlist check failed" >&2
  exit 1
fi

count="$(wc -l <"$EXPECTED_FILE" | tr -d ' ')"
echo "oxiland-capi exports match allowlist ($count symbols)"
