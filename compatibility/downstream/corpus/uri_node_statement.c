/*
 * Frozen 0.11 C corpus: URI, node, and statement basics.
 * Compiles against system Redland and Oxiland librdf.h under -Werror.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <librdf.h>

int main(void) {
  librdf_world *world = NULL;
  librdf_uri *uri = NULL;
  librdf_node *subject = NULL;
  librdf_node *predicate = NULL;
  librdf_node *object = NULL;
  librdf_node *blank = NULL;
  librdf_statement *statement = NULL;
  unsigned char *as_string = NULL;
  int rc = 1;

  world = librdf_new_world();
  if (!world)
    goto cleanup;
  librdf_world_open(world);

  uri = librdf_new_uri(world, (const unsigned char *)"http://example.org/term");
  if (!uri)
    goto cleanup;
  as_string = librdf_uri_to_string(uri);
  if (!as_string || strcmp((const char *)as_string, "http://example.org/term") != 0)
    goto cleanup;
  librdf_free_memory(as_string);
  as_string = NULL;

  /* Both Redland and Oxiland copy/borrow the URI; caller retains ownership. */
  subject = librdf_new_node_from_uri(world, uri);
  if (!subject || !librdf_node_is_resource(subject))
    goto cleanup;
  librdf_free_uri(uri);
  uri = NULL;

  predicate = librdf_new_node_from_uri_string(
      world, (const unsigned char *)"http://example.org/name");
  if (!predicate || !librdf_node_is_resource(predicate))
    goto cleanup;

  object = librdf_new_node_from_literal(world, (const unsigned char *)"hello", NULL, 0);
  if (!object || !librdf_node_is_literal(object))
    goto cleanup;

  blank = librdf_new_node_from_blank_identifier(world, (const unsigned char *)"b1");
  if (!blank || !librdf_node_is_blank(blank))
    goto cleanup;
  librdf_free_node(blank);
  blank = NULL;

  statement = librdf_new_statement_from_nodes(world, subject, predicate, object);
  subject = predicate = object = NULL;
  if (!statement)
    goto cleanup;
  if (!librdf_statement_get_subject(statement) ||
      !librdf_statement_get_predicate(statement) ||
      !librdf_statement_get_object(statement))
    goto cleanup;
  if (!librdf_statement_is_complete(statement))
    goto cleanup;

  rc = 0;
  puts("uri_node_statement ok");

cleanup:
  if (as_string)
    librdf_free_memory(as_string);
  if (statement)
    librdf_free_statement(statement);
  if (blank)
    librdf_free_node(blank);
  if (object)
    librdf_free_node(object);
  if (predicate)
    librdf_free_node(predicate);
  if (subject)
    librdf_free_node(subject);
  if (uri)
    librdf_free_uri(uri);
  if (world)
    librdf_free_world(world);
  if (rc != 0)
    fprintf(stderr, "uri_node_statement failed\n");
  return rc;
}
