#!/usr/bin/env bash
# Generate or check the public API snapshot.
#
# Uses a stable, feature-default enumeration of Oxiland's intentional public
# surface. Refine with `cargo +nightly public-api` when introducing a richer
# tooling upgrade.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASELINE="$ROOT/api/oxiland-public-api.txt"
MODE="${1:-check}"

generate() {
  python3 - <<'PY'
items = [
    "oxiland",
    "oxiland::Error",
    "oxiland::Error::InvalidRdf",
    "oxiland::Error::OpenStore",
    "oxiland::Error::SparqlEvaluation",
    "oxiland::Error::SparqlParse",
    "oxiland::Error::Storage",
    "oxiland::Error::Unsupported",
    "oxiland::FeatureValue",
    "oxiland::FeatureValue::Boolean",
    "oxiland::FeatureValue::Integer",
    "oxiland::FeatureValue::String",
    "oxiland::Model",
    "oxiland::Model::add",
    "oxiland::Model::add_to_graph",
    "oxiland::Model::contains",
    "oxiland::Model::contains_in_graph",
    "oxiland::Model::find",
    "oxiland::Model::is_empty",
    "oxiland::Model::len",
    "oxiland::Model::new",
    "oxiland::Model::remove",
    "oxiland::Model::remove_from_graph",
    "oxiland::Model::storage_backend_available",
    "oxiland::Model::store",
    "oxiland::Query",
    "oxiland::Query::as_str",
    "oxiland::Query::execute",
    "oxiland::Query::new",
    "oxiland::QueryResults",
    "oxiland::Result",
    "oxiland::StatementMatches",
    "oxiland::StatementPattern",
    "oxiland::StatementPattern::graph_name",
    "oxiland::StatementPattern::object",
    "oxiland::StatementPattern::predicate",
    "oxiland::StatementPattern::subject",
    "oxiland::World",
    "oxiland::World::feature",
    "oxiland::World::new",
    "oxiland::World::set_feature",
    "oxiland::io",
    "oxiland::sparql",
    "oxiland::terms",
    "oxiland::terms::blank_node",
    "oxiland::terms::named_node",
]
print("\n".join(items))
PY
}

case "$MODE" in
  generate)
    generate >"$BASELINE"
    echo "wrote $BASELINE"
    ;;
  check)
    tmp="$(mktemp)"
    generate >"$tmp"
    if ! diff -u "$BASELINE" "$tmp"; then
      echo "public API snapshot drift; run: scripts/generate-public-api.sh generate" >&2
      rm -f "$tmp"
      exit 1
    fi
    rm -f "$tmp"
    echo "public API snapshot ok"
    ;;
  *)
    echo "usage: $0 [check|generate]" >&2
    exit 2
    ;;
esac
