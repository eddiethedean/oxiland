#!/usr/bin/env bash
# Generate or check the public API snapshot.
#
# The baseline is a curated allowlist of Oxiland's intentional public surface.
# `check` diffs the embedded list against api/oxiland-public-api.txt, then runs
# scripts/check-public-api-owned.py so owned src/ items cannot drift silently.
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
    "oxiland::Error::Io",
    "oxiland::Error::OpenStore",
    "oxiland::Error::Parse",
    "oxiland::Error::Serialize",
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
    "oxiland::Model::insert_quad",
    "oxiland::Model::is_empty",
    "oxiland::Model::len",
    "oxiland::Model::new",
    "oxiland::Model::open",
    "oxiland::Model::remove",
    "oxiland::Model::remove_from_graph",
    "oxiland::Model::remove_quad",
    "oxiland::Model::storage_backend_available",
    "oxiland::Model::store",
    "oxiland::ParseError",
    "oxiland::ParseError::location",
    "oxiland::ParseError::message",
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
    "oxiland::io::GraphTarget",
    "oxiland::io::GraphTarget::Dataset",
    "oxiland::io::GraphTarget::DefaultGraph",
    "oxiland::io::GraphTarget::Named",
    "oxiland::io::Parser",
    "oxiland::io::Parser::base_iri",
    "oxiland::io::Parser::for_syntax",
    "oxiland::io::Parser::graph_target",
    "oxiland::io::Parser::load_collecting",
    "oxiland::io::Parser::load_into",
    "oxiland::io::Parser::load_path_collecting",
    "oxiland::io::Parser::load_path_into",
    "oxiland::io::Parser::parse_path",
    "oxiland::io::Parser::parse_path_with_extension",
    "oxiland::io::Parser::parse_reader",
    "oxiland::io::Parser::parse_slice",
    "oxiland::io::Parser::parse_str",
    "oxiland::io::Parser::syntax",
    "oxiland::io::QuadStream",
    "oxiland::io::Serializer",
    "oxiland::io::Serializer::base_iri",
    "oxiland::io::Serializer::for_syntax",
    "oxiland::io::Serializer::serialize_model_to_path",
    "oxiland::io::Serializer::serialize_model_to_string",
    "oxiland::io::Serializer::serialize_model_to_writer",
    "oxiland::io::Serializer::serialize_quads_to_string",
    "oxiland::io::Serializer::serialize_quads_to_writer",
    "oxiland::io::Serializer::serialize_triples_to_writer",
    "oxiland::io::Serializer::syntax",
    "oxiland::io::Serializer::with_prefix",
    "oxiland::io::SliceStream",
    "oxiland::io::SourceLocation",
    "oxiland::io::SourceLocation::column",
    "oxiland::io::SourceLocation::line",
    "oxiland::io::SourceLocation::offset",
    "oxiland::io::Syntax",
    "oxiland::io::Syntax::NQuads",
    "oxiland::io::Syntax::NTriples",
    "oxiland::io::Syntax::RdfXml",
    "oxiland::io::Syntax::TriG",
    "oxiland::io::Syntax::Turtle",
    "oxiland::io::Syntax::all",
    "oxiland::io::Syntax::can_parse",
    "oxiland::io::Syntax::can_serialize",
    "oxiland::io::Syntax::extension",
    "oxiland::io::Syntax::from_extension",
    "oxiland::io::Syntax::from_media_type",
    "oxiland::io::Syntax::from_name",
    "oxiland::io::Syntax::media_type",
    "oxiland::io::Syntax::name",
    "oxiland::io::Syntax::supports_datasets",
    "oxiland::io::primitives",
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
    python3 "$ROOT/scripts/check-public-api-owned.py"
    ;;
  *)
    echo "usage: $0 [check|generate]" >&2
    exit 2
    ;;
esac
