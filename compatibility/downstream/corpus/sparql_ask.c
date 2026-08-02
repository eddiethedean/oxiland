/*
 * Frozen 0.11 C corpus: SPARQL ASK against a parsed model.
 * Compiles against system Redland and Oxiland librdf.h under -Werror.
 */

#include <stdio.h>
#include <stdlib.h>

#include <librdf.h>

int main(void) {
  librdf_world *world = NULL;
  librdf_storage *storage = NULL;
  librdf_model *model = NULL;
  librdf_parser *parser = NULL;
  librdf_uri *base = NULL;
  librdf_query *query = NULL;
  librdf_query_results *results = NULL;
  const unsigned char *ttl =
      (const unsigned char *)"<http://example.org/s> <http://example.org/p> \"v\" .";
  int rc = 1;

  world = librdf_new_world();
  if (!world)
    goto cleanup;
  librdf_world_open(world);

  storage = librdf_new_storage(world, "memory", NULL, NULL);
  if (!storage)
    goto cleanup;

  model = librdf_new_model(world, storage, NULL);
  if (!model)
    goto cleanup;

  parser = librdf_new_parser(world, "turtle", NULL, NULL);
  if (!parser)
    goto cleanup;
  base = librdf_new_uri(world, (const unsigned char *)"http://example.org/");
  if (!base)
    goto cleanup;
  if (librdf_parser_parse_string_into_model(parser, ttl, base, model) != 0)
    goto cleanup;

  query = librdf_new_query(world, "sparql", NULL,
                           (const unsigned char *)"ASK { ?s ?p ?o }", NULL);
  if (!query)
    goto cleanup;

  results = librdf_model_query_execute(model, query);
  if (!results || !librdf_query_results_is_boolean(results) ||
      !librdf_query_results_get_boolean(results))
    goto cleanup;

  rc = 0;
  puts("sparql_ask ok");

cleanup:
  if (results)
    librdf_free_query_results(results);
  if (query)
    librdf_free_query(query);
  if (base)
    librdf_free_uri(base);
  if (parser)
    librdf_free_parser(parser);
  if (model)
    librdf_free_model(model);
  if (storage)
    librdf_free_storage(storage);
  if (world)
    librdf_free_world(world);
  if (rc != 0)
    fprintf(stderr, "sparql_ask failed\n");
  return rc;
}
