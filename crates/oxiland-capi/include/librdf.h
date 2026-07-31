#ifndef OXILAND_LIBRDF_H
#define OXILAND_LIBRDF_H

/**
 * Oxiland 0.8 C ABI preview — Redland-shaped allowlist.
 * Full ABI compatibility remains 0.9. See docs/design/0.8-cabi.md.
 */

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>

typedef struct librdf_world_s librdf_world;
typedef struct librdf_storage_s librdf_storage;
typedef struct librdf_model_s librdf_model;
typedef struct librdf_uri_s librdf_uri;
typedef struct librdf_node_s librdf_node;
typedef struct librdf_statement_s librdf_statement;
typedef struct librdf_stream_s librdf_stream;
typedef struct librdf_parser_s librdf_parser;
typedef struct librdf_serializer_s librdf_serializer;
typedef struct librdf_query_s librdf_query;
typedef struct librdf_query_results_s librdf_query_results;

/* World */
librdf_world *librdf_new_world(void);
void librdf_free_world(librdf_world *world);
void librdf_world_open(librdf_world *world);

/* Storage */
librdf_storage *librdf_new_storage(librdf_world *world, const char *storage_name,
                                   const char *name, const char *options);
void librdf_free_storage(librdf_storage *storage);
int librdf_storage_open(librdf_storage *storage, librdf_model *model);

/* Model */
librdf_model *librdf_new_model(librdf_world *world, librdf_storage *storage,
                               const char *options);
void librdf_free_model(librdf_model *model);
int librdf_model_add_statement(librdf_model *model, librdf_statement *statement);
int librdf_model_remove_statement(librdf_model *model, librdf_statement *statement);
int librdf_model_contains_statement(librdf_model *model, librdf_statement *statement);
int librdf_model_size(librdf_model *model);
librdf_stream *librdf_model_find_statements(librdf_model *model,
                                            librdf_statement *statement);

/* Terms */
librdf_uri *librdf_new_uri(librdf_world *world, const unsigned char *uri_string);
void librdf_free_uri(librdf_uri *uri);
librdf_node *librdf_new_node_from_uri_string(librdf_world *world,
                                             const unsigned char *uri_string);
librdf_node *librdf_new_node_from_literal(librdf_world *world,
                                          const unsigned char *string,
                                          const char *xml_language, int is_wf_xml);
void librdf_free_node(librdf_node *node);
librdf_statement *librdf_new_statement_from_nodes(librdf_world *world,
                                                  librdf_node *subject,
                                                  librdf_node *predicate,
                                                  librdf_node *object);
void librdf_free_statement(librdf_statement *statement);

/* Stream */
int librdf_stream_end(librdf_stream *stream);
int librdf_stream_next(librdf_stream *stream);
librdf_statement *librdf_stream_get_object(librdf_stream *stream);
void librdf_free_stream(librdf_stream *stream);

/* Parser */
librdf_parser *librdf_new_parser(librdf_world *world, const char *name,
                                 const char *mime_type, librdf_uri *type_uri);
void librdf_free_parser(librdf_parser *parser);
int librdf_parser_check_name(librdf_world *world, const char *name);
int librdf_parser_parse_string_into_model(librdf_parser *parser,
                                          const unsigned char *string,
                                          librdf_uri *base_uri, librdf_model *model);

/* Serializer */
librdf_serializer *librdf_new_serializer(librdf_world *world, const char *name,
                                         const char *mime_type, librdf_uri *type_uri);
void librdf_free_serializer(librdf_serializer *serializer);
int librdf_serializer_check_name(librdf_world *world, const char *name);
unsigned char *librdf_serializer_serialize_model_to_string(librdf_serializer *serializer,
                                                           librdf_uri *base_uri,
                                                           librdf_model *model);

/* Query */
librdf_query *librdf_new_query(librdf_world *world, const char *name, librdf_uri *uri,
                               const unsigned char *query_string, librdf_uri *query_uri);
void librdf_free_query(librdf_query *query);
librdf_query_results *librdf_model_query_execute(librdf_model *model, librdf_query *query);
int librdf_query_results_is_boolean(librdf_query_results *query_results);
int librdf_query_results_get_boolean(librdf_query_results *query_results);
int librdf_query_results_is_bindings(librdf_query_results *query_results);
int librdf_query_results_finished(librdf_query_results *query_results);
int librdf_query_results_next(librdf_query_results *query_results);
const char *librdf_query_results_get_binding_name(librdf_query_results *query_results,
                                                  int offset);
librdf_node *librdf_query_results_get_binding_value(librdf_query_results *query_results,
                                                    int offset);
int librdf_query_results_get_bindings_count(librdf_query_results *query_results);
void librdf_free_query_results(librdf_query_results *query_results);

/* Alloc */
void librdf_free_memory(void *ptr);

#ifdef __cplusplus
}
#endif

#endif /* OXILAND_LIBRDF_H */
