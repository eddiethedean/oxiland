/*
 * Frozen 0.11 C corpus: parse Turtle string into a model.
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
  if (librdf_model_size(model) < 1)
    goto cleanup;

  rc = 0;
  puts("turtle_parse_string ok");

cleanup:
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
    fprintf(stderr, "turtle_parse_string failed\n");
  return rc;
}
