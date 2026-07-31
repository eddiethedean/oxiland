#ifndef OXILAND_LIBRDF_H
#define OXILAND_LIBRDF_H

/**
 * Oxiland 0.9 C ABI — Redland-shaped surface beyond the 0.8 preview.
 * See docs/design/0.9-cabi.md. Source-compat + Oxiland ABI; not Redland .so swap.
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
typedef struct librdf_digest_s librdf_digest;

/* Node type constants (Redland-compatible) */
#define LIBRDF_NODE_TYPE_UNKNOWN  0
#define LIBRDF_NODE_TYPE_RESOURCE 1
#define LIBRDF_NODE_TYPE_LITERAL  2
#define LIBRDF_NODE_TYPE_BLANK    4

typedef int (*librdf_log_func)(void *user_data, int code, int level, int facility,
                               const char *message, const char *locator);

/* World */
librdf_world *librdf_new_world(void);
void librdf_free_world(librdf_world *world);
void librdf_world_open(librdf_world *world);
int librdf_world_set_logger(librdf_world *world, void *user_data, librdf_log_func logger);
void librdf_log_simple(librdf_world *world, int code, int level, int facility,
                       const char *message);

/* Storage */
librdf_storage *librdf_new_storage(librdf_world *world, const char *storage_name,
                                   const char *name, const char *options);
void librdf_free_storage(librdf_storage *storage);
int librdf_storage_open(librdf_storage *storage, librdf_model *model);
int librdf_storage_enumerate(librdf_world *world, unsigned int counter, const char **name,
                             const char **label);
int librdf_storage_sync(librdf_storage *storage);

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
int librdf_model_sync(librdf_model *model);
librdf_stream *librdf_model_as_stream(librdf_model *model);
int librdf_model_add(librdf_model *model, librdf_node *subject, librdf_node *predicate,
                     librdf_node *object);
unsigned char *librdf_model_to_string(librdf_model *model, librdf_uri *base_uri);
int librdf_model_update(librdf_model *model, const unsigned char *update_string);
librdf_query_results *librdf_model_query_execute(librdf_model *model, librdf_query *query);

/* Terms */
librdf_uri *librdf_new_uri(librdf_world *world, const unsigned char *uri_string);
void librdf_free_uri(librdf_uri *uri);
const char *librdf_uri_as_string(librdf_uri *uri);
unsigned char *librdf_uri_to_string(librdf_uri *uri);
int librdf_uri_equals(librdf_uri *first_uri, librdf_uri *second_uri);
int librdf_uri_compare(librdf_uri *first_uri, librdf_uri *second_uri);
int librdf_uri_is_file_uri(librdf_uri *uri);
char *librdf_uri_to_filename(librdf_uri *uri);

librdf_node *librdf_new_node_from_uri_string(librdf_world *world,
                                             const unsigned char *uri_string);
librdf_node *librdf_new_node_from_literal(librdf_world *world,
                                          const unsigned char *string,
                                          const char *xml_language, int is_wf_xml);
librdf_node *librdf_new_node_from_blank_identifier(librdf_world *world,
                                                   const unsigned char *identifier);
void librdf_free_node(librdf_node *node);
int librdf_node_get_type(librdf_node *node);
int librdf_node_is_resource(librdf_node *node);
int librdf_node_is_literal(librdf_node *node);
int librdf_node_is_blank(librdf_node *node);
librdf_uri *librdf_node_get_uri(librdf_node *node);
const char *librdf_node_get_literal_value(librdf_node *node);
const char *librdf_node_get_literal_value_language(librdf_node *node);
const char *librdf_node_get_blank_identifier(librdf_node *node);
unsigned char *librdf_node_to_string(librdf_node *node);
int librdf_node_equals(librdf_node *first_node, librdf_node *second_node);

librdf_statement *librdf_new_statement(librdf_world *world);
librdf_statement *librdf_new_statement_from_nodes(librdf_world *world,
                                                  librdf_node *subject,
                                                  librdf_node *predicate,
                                                  librdf_node *object);
void librdf_free_statement(librdf_statement *statement);
librdf_node *librdf_statement_get_subject(librdf_statement *statement);
librdf_node *librdf_statement_get_predicate(librdf_statement *statement);
librdf_node *librdf_statement_get_object(librdf_statement *statement);
void librdf_statement_set_subject(librdf_statement *statement, librdf_node *node);
void librdf_statement_set_predicate(librdf_statement *statement, librdf_node *node);
void librdf_statement_set_object(librdf_statement *statement, librdf_node *node);
int librdf_statement_equals(librdf_statement *first, librdf_statement *second);
int librdf_statement_is_complete(librdf_statement *statement);
unsigned char *librdf_statement_to_string(librdf_statement *statement);

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
int librdf_parser_parse_counted_string_into_model(librdf_parser *parser,
                                                  const unsigned char *string,
                                                  size_t length, librdf_uri *base_uri,
                                                  librdf_model *model);

/* Serializer */
librdf_serializer *librdf_new_serializer(librdf_world *world, const char *name,
                                         const char *mime_type, librdf_uri *type_uri);
void librdf_free_serializer(librdf_serializer *serializer);
int librdf_serializer_check_name(librdf_world *world, const char *name);
unsigned char *librdf_serializer_serialize_model_to_string(librdf_serializer *serializer,
                                                           librdf_uri *base_uri,
                                                           librdf_model *model);
unsigned char *librdf_serializer_serialize_model_to_counted_string(
    librdf_serializer *serializer, librdf_uri *base_uri, librdf_model *model,
    size_t *length_p);
int librdf_serializer_serialize_model_to_file(librdf_serializer *serializer,
                                              const char *name, librdf_uri *base_uri,
                                              librdf_model *model);

/* Query */
librdf_query *librdf_new_query(librdf_world *world, const char *name, librdf_uri *uri,
                               const unsigned char *query_string, librdf_uri *query_uri);
void librdf_free_query(librdf_query *query);
int librdf_query_results_is_boolean(librdf_query_results *query_results);
int librdf_query_results_get_boolean(librdf_query_results *query_results);
int librdf_query_results_is_bindings(librdf_query_results *query_results);
int librdf_query_results_is_graph(librdf_query_results *query_results);
librdf_stream *librdf_query_results_as_stream(librdf_query_results *query_results);
int librdf_query_results_finished(librdf_query_results *query_results);
int librdf_query_results_next(librdf_query_results *query_results);
const char *librdf_query_results_get_binding_name(librdf_query_results *query_results,
                                                  int offset);
librdf_node *librdf_query_results_get_binding_value(librdf_query_results *query_results,
                                                    int offset);
int librdf_query_results_get_bindings_count(librdf_query_results *query_results);
void librdf_free_query_results(librdf_query_results *query_results);

/* Digests */
librdf_digest *librdf_new_digest(librdf_world *world, const char *name);
void librdf_free_digest(librdf_digest *digest);
void librdf_digest_init(librdf_digest *digest);
void librdf_digest_update(librdf_digest *digest, const unsigned char *buffer, size_t length);
void librdf_digest_update_string(librdf_digest *digest, const unsigned char *string);
void librdf_digest_final(librdf_digest *digest);
char *librdf_digest_to_string(librdf_digest *digest);
unsigned char *librdf_digest_get_digest(librdf_digest *digest);
size_t librdf_digest_get_digest_length(librdf_digest *digest);

/* UTF-8 / path helpers */
unsigned char *librdf_utf8_to_latin1(const unsigned char *input, size_t length,
                                     size_t *output_length);
unsigned char *librdf_latin1_to_utf8(const unsigned char *input, size_t length,
                                     size_t *output_length);
char *librdf_basename(const char *name);

/* Alloc */
void librdf_free_memory(void *ptr);

#ifdef __cplusplus
}
#endif

#endif /* OXILAND_LIBRDF_H */
