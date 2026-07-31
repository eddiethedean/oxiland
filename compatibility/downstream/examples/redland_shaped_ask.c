#include <stdio.h>
#include <stdlib.h>
#include "librdf.h"

/* Selected Redland-shaped ASK workflow rebuilt against oxiland-capi. */
int main(void) {
  librdf_world *world = librdf_new_world();
  librdf_world_open(world);
  librdf_storage *storage = librdf_new_storage(world, "memory", NULL, NULL);
  librdf_model *model = librdf_new_model(world, storage, NULL);
  librdf_parser *parser = librdf_new_parser(world, "turtle", NULL, NULL);
  const unsigned char *ttl =
      (const unsigned char *)"<http://example.org/s> <http://example.org/p> \"v\" .";
  if (librdf_parser_parse_string_into_model(parser, ttl, NULL, model) != 0) {
    fprintf(stderr, "parse failed\n");
    return 1;
  }
  librdf_query *query = librdf_new_query(
      world, "sparql", NULL, (const unsigned char *)"ASK { ?s ?p ?o }", NULL);
  librdf_query_results *results = librdf_model_query_execute(model, query);
  if (!results || !librdf_query_results_is_boolean(results) ||
      !librdf_query_results_get_boolean(results)) {
    fprintf(stderr, "ASK expected true\n");
    return 1;
  }
  librdf_free_query_results(results);
  librdf_free_query(query);
  librdf_free_parser(parser);
  librdf_free_model(model);
  librdf_free_storage(storage);
  librdf_free_world(world);
  puts("redland_shaped_ask ok");
  return 0;
}
