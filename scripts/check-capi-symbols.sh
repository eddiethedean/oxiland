#!/usr/bin/env bash
# Verify oxiland-capi exports exactly the 0.8 preview allowlist.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ALLOWLIST="$ROOT/crates/oxiland-capi/symbols.version"
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
echo "oxiland-capi exports match preview allowlist ($count symbols)"
