#!/usr/bin/env bash
# Downstream packaging + differential smokes for Oxiland 0.9.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

cargo build -p oxiland-capi --locked
scripts/check-capi-symbols.sh debug

# Installed-artifact smoke: stage headers/libs outside the workspace tree.
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
mkdir -p "$STAGE/include" "$STAGE/lib" "$STAGE/lib/pkgconfig"
cp crates/oxiland-capi/include/librdf.h "$STAGE/include/"
cp target/debug/liboxiland_capi.* "$STAGE/lib/" 2>/dev/null || true
# Select the platform-specific shared library produced by Cargo. Do not leave
# missing candidates as literal array entries: CI runs this smoke on Linux.
LIB=""
for candidate in target/debug/liboxiland_capi.so target/debug/liboxiland_capi.dylib; do
  if [[ -f "$candidate" ]]; then
    LIB="$candidate"
    break
  fi
done
if [[ -z "$LIB" ]]; then
  echo "error: built shared library missing" >&2
  exit 1
fi
cp "$LIB" "$STAGE/lib/"
PREFIX="$STAGE"
sed "s|@PREFIX@|$PREFIX|g; s|@VERSION@|0.11.0|g" crates/oxiland-capi/oxiland.pc.in \
  > "$STAGE/lib/pkgconfig/oxiland.pc"

cat > "$STAGE/smoke.c" <<'EOF'
#include <stdio.h>
#include "librdf.h"
int main(void) {
  librdf_world *w = librdf_new_world();
  if (!w) return 1;
  librdf_world_open(w);
  librdf_free_world(w);
  puts("oxiland-capi installed-artifact smoke ok");
  return 0;
}
EOF

cc -I "$STAGE/include" -L "$STAGE/lib" "$STAGE/smoke.c" -loxiland_capi \
  -Wl,-rpath,"$STAGE/lib" -o "$STAGE/smoke"
"$STAGE/smoke"

# Differential fixture: ASK true/false parity shape (Oxiland-only when Redland absent).
python3 compatibility/downstream/differential/run_oxiland_fixtures.py

# Redland-shaped example rebuild
cc -I crates/oxiland-capi/include -L target/debug \
  compatibility/downstream/examples/redland_shaped_ask.c \
  -loxiland_capi -Wl,-rpath,"$PWD/target/debug" \
  -o target/debug/redland_shaped_ask
target/debug/redland_shaped_ask

# Ruby binding smoke is optional when ruby/dev headers are absent.
if command -v ruby >/dev/null 2>&1; then
  echo "ruby present; see compatibility/downstream/ruby-smoke/README.md"
else
  echo "ruby not installed; skipping binding smoke (D-09-01)"
fi

echo "downstream smokes passed"
