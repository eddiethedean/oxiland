#!/usr/bin/env bash
# Diff the live Python PEP 561 stub against the frozen api/ snapshot.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LIVE="$ROOT/python/oxiland.pyi"
FROZEN="$ROOT/api/oxiland-python.pyi"

if [[ ! -f "$LIVE" ]]; then
  echo "error: missing live stub $LIVE" >&2
  exit 1
fi

if [[ ! -f "$FROZEN" ]]; then
  echo "error: missing frozen stub snapshot $FROZEN" >&2
  exit 1
fi

if ! diff -u "$FROZEN" "$LIVE"; then
  echo "error: python/oxiland.pyi differs from api/oxiland-python.pyi" >&2
  echo "Update the frozen snapshot intentionally after reviewing the stub change:" >&2
  echo "  cp python/oxiland.pyi api/oxiland-python.pyi" >&2
  exit 1
fi

echo "Python stub snapshot matches api/oxiland-python.pyi"
