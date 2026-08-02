/*
 * Frozen 0.11 C corpus: memory storage, model add, and size.
 * Compiles against system Redland and Oxiland librdf.h under -Werror.
 */

#include <stdio.h>
#include <stdlib.h>

#include <librdf.h>

int main(void) {
  librdf_world *world = NULL;
  librdf_storage *storage = NULL;
  librdf_model *model = NULL;
  librdf_node *s = NULL;
  librdf_node *p = NULL;
  librdf_node *o = NULL;
  librdf_statement *statement = NULL;
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

  s = librdf_new_node_from_uri_string(world, (const unsigned char *)"http://example.org/s");
  p = librdf_new_node_from_uri_string(world, (const unsigned char *)"http://example.org/p");
  o = librdf_new_node_from_literal(world, (const unsigned char *)"v", NULL, 0);
  if (!s || !p || !o)
    goto cleanup;

  statement = librdf_new_statement_from_nodes(world, s, p, o);
  s = p = o = NULL;
  if (!statement)
    goto cleanup;

  if (librdf_model_add_statement(model, statement) != 0)
    goto cleanup;
  if (librdf_model_size(model) < 1)
    goto cleanup;

  rc = 0;
  puts("model_memory_add_size ok");

cleanup:
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
  if (rc != 0)
    fprintf(stderr, "model_memory_add_size failed\n");
  return rc;
}
