#!/usr/bin/env bash
# Build and run the frozen 0.11 C source corpus against system Redland and Oxiland.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"

CORPUS_DIR="$ROOT/compatibility/downstream/corpus"
OUT_DIR="${CORPUS_OUT_DIR:-$ROOT/target/corpus}"
mkdir -p "$OUT_DIR/redland" "$OUT_DIR/oxiland"

SOURCES=(
  world_open_close.c
  model_memory_add_size.c
  turtle_parse_string.c
  sparql_ask.c
  uri_node_statement.c
)

if ! command -v pkg-config >/dev/null 2>&1; then
  echo "error: pkg-config is required" >&2
  exit 1
fi
if ! pkg-config --exists redland; then
  echo "error: redland.pc not found (install Redland 1.0.17 development package)" >&2
  exit 1
fi

REDLAND_CFLAGS="$(pkg-config --cflags redland)"
REDLAND_LIBS="$(pkg-config --libs redland)"

echo "==> Building corpus against system Redland"
for src in "${SOURCES[@]}"; do
  name="${src%.c}"
  echo "  cc -Werror [redland] $src"
  # shellcheck disable=SC2086
  cc -std=c11 -Wall -Wextra -Werror $REDLAND_CFLAGS \
    "$CORPUS_DIR/$src" $REDLAND_LIBS \
    -o "$OUT_DIR/redland/$name"
done

echo "==> Running Redland corpus binaries"
for src in "${SOURCES[@]}"; do
  name="${src%.c}"
  "$OUT_DIR/redland/$name"
done

echo "==> Building oxiland-capi (release)"
cargo build -p oxiland-capi --release --locked

OXILAND_INCLUDE="$ROOT/crates/oxiland-capi/include"
OXILAND_LIBDIR="$ROOT/target/release"
OXILAND_EXTRA=()
case "$(uname -s)" in
  Darwin)
    OXILAND_EXTRA+=(-Wl,-rpath,"$OXILAND_LIBDIR")
    ;;
  Linux)
    OXILAND_EXTRA+=(-Wl,-rpath,"$OXILAND_LIBDIR")
    ;;
esac

echo "==> Building corpus against Oxiland"
for src in "${SOURCES[@]}"; do
  name="${src%.c}"
  echo "  cc -Werror [oxiland] $src"
  cc -std=c11 -Wall -Wextra -Werror \
    -I "$OXILAND_INCLUDE" \
    -L "$OXILAND_LIBDIR" \
    "$CORPUS_DIR/$src" \
    -loxiland_capi \
    "${OXILAND_EXTRA[@]}" \
    -o "$OUT_DIR/oxiland/$name"
done

echo "==> Running Oxiland corpus binaries"
for src in "${SOURCES[@]}"; do
  name="${src%.c}"
  "$OUT_DIR/oxiland/$name"
done

echo "corpus: all ${#SOURCES[@]} programs passed (Redland + Oxiland)"
