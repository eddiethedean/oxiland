/*
 * Representative Redland-shaped workflow for Oxiland 0.8 C ABI preview.
 *
 * Build (from repo root, after `cargo build -p oxiland-capi`):
 *
 *   cc -I crates/oxiland-capi/include -L target/debug \
 *      crates/oxiland-capi/examples/preview_workflow.c \
 *      -loxiland_capi -o preview_workflow
 *
 * On macOS add: -Wl,-rpath,$(pwd)/target/debug
 */

#include "librdf.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int fail(const char *msg) {
  fprintf(stderr, "preview_workflow: %s\n", msg);
  return 1;
}

int main(void) {
  librdf_world *world = NULL;
  librdf_storage *storage = NULL;
  librdf_model *model = NULL;
  librdf_node *s = NULL, *p = NULL, *o = NULL;
  librdf_statement *statement = NULL;
  librdf_statement *pattern = NULL;
  librdf_stream *stream = NULL;
  librdf_parser *parser = NULL;
  librdf_serializer *serializer = NULL;
  librdf_query *ask = NULL;
  librdf_query *selectq = NULL;
  librdf_query_results *results = NULL;
  unsigned char *serialized = NULL;
  int rc = 1;

  world = librdf_new_world();
  if (!world)
    return fail("librdf_new_world");
  librdf_world_open(world);

  storage = librdf_new_storage(world, "memory", NULL, NULL);
  if (!storage)
    return fail("librdf_new_storage");

  model = librdf_new_model(world, storage, NULL);
  if (!model)
    return fail("librdf_new_model");

  s = librdf_new_node_from_uri_string(world, (const unsigned char *)"http://example.org/alice");
  p = librdf_new_node_from_uri_string(world, (const unsigned char *)"http://example.org/name");
  o = librdf_new_node_from_literal(world, (const unsigned char *)"Alice", NULL, 0);
  if (!s || !p || !o)
    goto cleanup;

  statement = librdf_new_statement_from_nodes(world, s, p, o);
  s = p = o = NULL; /* ownership transferred */
  if (!statement)
    goto cleanup;

  if (librdf_model_add_statement(model, statement) != 0)
    goto cleanup;
  if (!librdf_model_contains_statement(model, statement))
    goto cleanup;
  if (librdf_model_size(model) < 1)
    goto cleanup;

  pattern = librdf_new_statement_from_nodes(world, NULL, NULL, NULL);
  if (!pattern)
    goto cleanup;
  stream = librdf_model_find_statements(model, pattern);
  if (!stream)
    goto cleanup;
  if (librdf_stream_end(stream))
    goto cleanup;
  if (!librdf_stream_get_object(stream))
    goto cleanup;
  librdf_free_stream(stream);
  stream = NULL;

  parser = librdf_new_parser(world, "turtle", NULL, NULL);
  if (!parser)
    goto cleanup;
  if (!librdf_parser_check_name(world, "turtle"))
    goto cleanup;
  if (librdf_parser_parse_string_into_model(
          parser,
          (const unsigned char *)"<http://example.org/bob> <http://example.org/name> \"Bob\" .",
          NULL, model) != 0)
    goto cleanup;

  serializer = librdf_new_serializer(world, "turtle", NULL, NULL);
  if (!serializer)
    goto cleanup;
  serialized = librdf_serializer_serialize_model_to_string(serializer, NULL, model);
  if (!serialized)
    goto cleanup;
  if (strstr((const char *)serialized, "Alice") == NULL)
    goto cleanup;
  librdf_free_memory(serialized);
  serialized = NULL;

  ask = librdf_new_query(world, "sparql", NULL,
                         (const unsigned char *)"ASK { ?s ?p ?o }", NULL);
  if (!ask)
    goto cleanup;
  results = librdf_model_query_execute(model, ask);
  if (!results || !librdf_query_results_is_boolean(results) ||
      !librdf_query_results_get_boolean(results))
    goto cleanup;
  librdf_free_query_results(results);
  results = NULL;

  selectq = librdf_new_query(
      world, "sparql", NULL,
      (const unsigned char *)"SELECT ?name WHERE { ?s <http://example.org/name> ?name }",
      NULL);
  if (!selectq)
    goto cleanup;
  results = librdf_model_query_execute(model, selectq);
  if (!results || !librdf_query_results_is_bindings(results) ||
      librdf_query_results_finished(results) ||
      librdf_query_results_get_bindings_count(results) < 1)
    goto cleanup;
  if (!librdf_query_results_get_binding_name(results, 0))
    goto cleanup;
  if (!librdf_query_results_get_binding_value(results, 0))
    goto cleanup;

  rc = 0;

cleanup:
  if (serialized)
    librdf_free_memory(serialized);
  if (results)
    librdf_free_query_results(results);
  if (selectq)
    librdf_free_query(selectq);
  if (ask)
    librdf_free_query(ask);
  if (serializer)
    librdf_free_serializer(serializer);
  if (parser)
    librdf_free_parser(parser);
  if (stream)
    librdf_free_stream(stream);
  if (pattern)
    librdf_free_statement(pattern);
  if (statement)
    librdf_free_statement(statement);
  if (o)
    librdf_free_node(o);
  if (p)
    librdf_free_node(p);
  if (s)
    librdf_free_node(s);
  if (model)
    librdf_free_model(model);
  if (storage)
    librdf_free_storage(storage);
  if (world)
    librdf_free_world(world);
  return rc;
}
