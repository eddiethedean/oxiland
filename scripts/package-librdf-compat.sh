#!/usr/bin/env bash
# Package oxiland-capi as a Redland-compatible shared library (librdf soname).
#
# After `cargo build -p oxiland-capi --release`, produces:
#   macOS: target/release/compat/librdf.0.dylib + librdf.dylib symlink
#   Linux: target/release/compat/librdf.so.0 + librdf.so symlink
#   Windows (best-effort): target/release/compat/librdf-0.dll
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PROFILE="${1:-release}"
LIBDIR="$ROOT/target/$PROFILE"
COMPAT="$LIBDIR/compat"
VERSION="${OXILAND_CAPI_VERSION:-0.13.0}"

if [[ ! -d "$LIBDIR" ]]; then
  echo "error: missing $LIBDIR (run: cargo build -p oxiland-capi --$PROFILE)" >&2
  exit 1
fi

mkdir -p "$COMPAT"
rm -f "$COMPAT"/librdf* "$COMPAT"/liboxiland_capi*

OS="$(uname -s)"
case "$OS" in
  Darwin)
    SRC=""
    for candidate in "$LIBDIR/liboxiland_capi.dylib"; do
      if [[ -f "$candidate" ]]; then
        SRC="$candidate"
        break
      fi
    done
    if [[ -z "$SRC" ]]; then
      echo "error: liboxiland_capi.dylib not found under $LIBDIR" >&2
      exit 1
    fi
    DEST="$COMPAT/librdf.0.dylib"
    cp "$SRC" "$DEST"
    install_name_tool -id "@rpath/librdf.0.dylib" "$DEST"
    ln -sf librdf.0.dylib "$COMPAT/librdf.dylib"
    echo "packaged $DEST (id @rpath/librdf.0.dylib)"
    echo "symlink  $COMPAT/librdf.dylib -> librdf.0.dylib"
    ;;
  Linux)
    SRC=""
    for candidate in "$LIBDIR/liboxiland_capi.so"; do
      if [[ -f "$candidate" ]]; then
        SRC="$candidate"
        break
      fi
    done
    if [[ -z "$SRC" ]]; then
      echo "error: liboxiland_capi.so not found under $LIBDIR" >&2
      exit 1
    fi
    DEST="$COMPAT/librdf.so.0"
    cp "$SRC" "$DEST"
    if command -v patchelf >/dev/null 2>&1; then
      patchelf --set-soname librdf.so.0 "$DEST"
      echo "packaged $DEST (soname librdf.so.0 via patchelf)"
    else
      cat >&2 <<'EOF'
warning: patchelf not found; librdf.so.0 copied without SONAME rewrite.
  Install patchelf, or link consumers with a linker script / -Wl,-soname,librdf.so.0
  when rebuilding the Oxiland cdylib directly.
EOF
      echo "packaged $DEST (soname unchanged; patchelf recommended)"
    fi
    ln -sf librdf.so.0 "$COMPAT/librdf.so"
    echo "symlink  $COMPAT/librdf.so -> librdf.so.0"
    ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT)
    SRC=""
    for candidate in \
      "$LIBDIR/oxiland_capi.dll" \
      "$LIBDIR/liboxiland_capi.dll" \
      "$LIBDIR/oxiland_capi.dll" \
      "$LIBDIR/deps/oxiland_capi.dll"
    do
      if [[ -f "$candidate" ]]; then
        SRC="$candidate"
        break
      fi
    done
    if [[ -z "$SRC" ]]; then
      echo "error: oxiland_capi.dll not found under $LIBDIR" >&2
      ls -la "$LIBDIR"/*.dll 2>/dev/null || true
      ls -la "$LIBDIR" | head -50 >&2 || true
      exit 1
    fi
    DEST="$COMPAT/librdf-0.dll"
    # Windows may lock a previously packaged DLL; write beside then replace.
    TMP="$COMPAT/librdf-0.dll.new"
    cp "$SRC" "$TMP"
    mv -f "$TMP" "$DEST"
    cp "$SRC" "$COMPAT/oxiland_capi.dll" 2>/dev/null || true
    echo "packaged $DEST"
    ;;
  *)
    # Git Bash / atypical uname: attempt Windows layout before giving up.
    if ls "$LIBDIR"/oxiland_capi.dll "$LIBDIR"/liboxiland_capi.dll >/dev/null 2>&1; then
      SRC="$(ls "$LIBDIR"/oxiland_capi.dll "$LIBDIR"/liboxiland_capi.dll 2>/dev/null | head -1)"
      DEST="$COMPAT/librdf-0.dll"
      cp "$SRC" "$DEST"
      echo "packaged $DEST (fallback OS=$OS)"
    else
      echo "error: unsupported OS: $OS" >&2
      exit 1
    fi
    ;;
esac

# Stage drop-in pkg-config next to the compat libs for local testing.
cat >"$COMPAT/redland.pc" <<EOF
prefix=$COMPAT
exec_prefix=\${prefix}
libdir=$COMPAT
includedir=$ROOT/crates/oxiland-capi/include

Name: Redland
Description: Oxiland drop-in Redland-compatible C ABI
Version: $VERSION
Libs: -L\${libdir} -lrdf
Cflags: -I\${includedir}
EOF
echo "wrote $COMPAT/redland.pc (from librdf-compat.pc.in layout)"

echo "compat packaging complete under $COMPAT"
