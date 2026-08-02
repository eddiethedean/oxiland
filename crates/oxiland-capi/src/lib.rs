//! C ABI for Oxiland (Redland-shaped source-compat).
//!
//! All `unsafe` lives here; the safe `oxiland` crate forbids it (ADR-002).

#![allow(non_camel_case_types)]
#![allow(missing_docs)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::redundant_closure)]

mod alloc;
mod error;
mod handles;

pub use alloc::{librdf_alloc_memory, librdf_calloc_memory, librdf_free_memory};

pub use handles::concepts::{
    librdf_get_concept_ms_namespace, librdf_get_concept_resource_by_index,
    librdf_get_concept_schema_namespace, librdf_get_concept_uri_by_index,
};

pub use handles::digest::{
    librdf_digest_final, librdf_digest_get_digest, librdf_digest_get_digest_length,
    librdf_digest_init, librdf_digest_print, librdf_digest_to_string, librdf_digest_update,
    librdf_digest_update_string, librdf_free_digest, librdf_new_digest,
};

pub use handles::files::librdf_files_temporary_file_name;

pub use handles::hash::{
    librdf_free_hash, librdf_hash_from_string, librdf_hash_get, librdf_hash_get_as_boolean,
    librdf_hash_get_as_long, librdf_hash_get_del, librdf_hash_interpret_template,
    librdf_hash_print, librdf_hash_print_keys, librdf_hash_print_values, librdf_hash_put_strings,
    librdf_hash_to_string, librdf_new_hash, librdf_new_hash_from_array_of_strings,
    librdf_new_hash_from_hash, librdf_new_hash_from_string,
};

pub use handles::helpers::{
    librdf_basename, librdf_latin1_to_utf8, librdf_latin1_to_utf8_2, librdf_unicode_char_to_utf8,
    librdf_utf8_print, librdf_utf8_to_latin1, librdf_utf8_to_latin1_2, librdf_utf8_to_unicode_char,
};

pub use handles::io::{
    oxiland_free_iostream, oxiland_iostream_data, oxiland_new_iostream,
    oxiland_new_iostream_from_bytes,
};

pub use handles::heuristics::{
    librdf_heuristic_gen_name, librdf_heuristic_get_blank_node, librdf_heuristic_is_blank_node,
    librdf_heuristic_object_is_literal,
};

pub use handles::iterator::{
    librdf_free_iterator, librdf_iterator_add_map, librdf_iterator_end,
    librdf_iterator_get_context, librdf_iterator_get_key, librdf_iterator_get_object,
    librdf_iterator_get_value, librdf_iterator_have_elements, librdf_iterator_next,
    librdf_new_empty_iterator, librdf_new_iterator,
};

pub use handles::list::{
    librdf_free_list, librdf_list_add, librdf_list_clear, librdf_list_contains,
    librdf_list_foreach, librdf_list_get_iterator, librdf_list_pop, librdf_list_remove,
    librdf_list_set_equals, librdf_list_shift, librdf_list_size, librdf_list_unshift,
    librdf_new_list,
};

pub use handles::log_msg::{
    librdf_log, librdf_log_message_code, librdf_log_message_facility, librdf_log_message_level,
    librdf_log_message_locator, librdf_log_message_message,
};

pub use handles::model::{
    librdf_free_model, librdf_model_add, librdf_model_add_statement, librdf_model_add_statements,
    librdf_model_add_string_literal_statement, librdf_model_add_submodel,
    librdf_model_add_typed_literal_statement, librdf_model_as_stream,
    librdf_model_contains_context, librdf_model_contains_statement,
    librdf_model_context_add_statement, librdf_model_context_add_statements,
    librdf_model_context_as_stream, librdf_model_context_remove_statement,
    librdf_model_context_remove_statements, librdf_model_context_serialize, librdf_model_enumerate,
    librdf_model_find_statements, librdf_model_find_statements_in_context,
    librdf_model_find_statements_with_options, librdf_model_get_arc, librdf_model_get_arcs,
    librdf_model_get_arcs_in, librdf_model_get_arcs_out, librdf_model_get_contexts,
    librdf_model_get_feature, librdf_model_get_source, librdf_model_get_sources,
    librdf_model_get_storage, librdf_model_get_target, librdf_model_get_targets,
    librdf_model_has_arc_in, librdf_model_has_arc_out, librdf_model_load, librdf_model_print,
    librdf_model_remove_statement, librdf_model_remove_submodel, librdf_model_serialise,
    librdf_model_set_feature, librdf_model_size, librdf_model_supports_contexts, librdf_model_sync,
    librdf_model_to_counted_string, librdf_model_to_string, librdf_model_transaction_commit,
    librdf_model_transaction_get_handle, librdf_model_transaction_rollback,
    librdf_model_transaction_start, librdf_model_transaction_start_with_handle,
    librdf_model_update, librdf_model_write, librdf_new_model, librdf_new_model_from_model,
    librdf_new_model_with_options,
};

pub use handles::node::{
    librdf_free_node, librdf_new_node, librdf_new_node_from_blank_identifier,
    librdf_new_node_from_counted_blank_identifier, librdf_new_node_from_counted_uri_string,
    librdf_new_node_from_literal, librdf_new_node_from_node,
    librdf_new_node_from_normalised_uri_string, librdf_new_node_from_typed_counted_literal,
    librdf_new_node_from_typed_literal, librdf_new_node_from_uri,
    librdf_new_node_from_uri_local_name, librdf_new_node_from_uri_string, librdf_node_decode,
    librdf_node_encode, librdf_node_equals, librdf_node_get_blank_identifier,
    librdf_node_get_counted_blank_identifier, librdf_node_get_li_ordinal,
    librdf_node_get_literal_value, librdf_node_get_literal_value_as_counted_string,
    librdf_node_get_literal_value_as_latin1, librdf_node_get_literal_value_datatype_uri,
    librdf_node_get_literal_value_is_wf_xml, librdf_node_get_literal_value_language,
    librdf_node_get_type, librdf_node_get_uri, librdf_node_is_blank, librdf_node_is_literal,
    librdf_node_is_resource, librdf_node_new_static_node_iterator, librdf_node_print,
    librdf_node_static_iterator_create, librdf_node_to_counted_string, librdf_node_to_string,
    librdf_node_write,
};

pub use handles::parser::{
    librdf_free_parser, librdf_new_parser, librdf_new_parser_from_factory,
    librdf_parser_check_name, librdf_parser_enumerate, librdf_parser_get_accept_header,
    librdf_parser_get_description, librdf_parser_get_feature,
    librdf_parser_get_namespaces_seen_count, librdf_parser_get_namespaces_seen_prefix,
    librdf_parser_get_namespaces_seen_uri, librdf_parser_get_uri_filter, librdf_parser_guess_name,
    librdf_parser_guess_name2, librdf_parser_parse_as_stream,
    librdf_parser_parse_counted_string_as_stream, librdf_parser_parse_counted_string_into_model,
    librdf_parser_parse_file_handle_as_stream, librdf_parser_parse_file_handle_into_model,
    librdf_parser_parse_into_model, librdf_parser_parse_iostream_as_stream,
    librdf_parser_parse_iostream_into_model, librdf_parser_parse_string_as_stream,
    librdf_parser_parse_string_into_model, librdf_parser_register_factory, librdf_parser_set_error,
    librdf_parser_set_feature, librdf_parser_set_uri_filter, librdf_parser_set_warning,
};

pub use handles::query::{
    librdf_free_query, librdf_free_query_results, librdf_free_query_results_formatter,
    librdf_model_query_execute, librdf_new_query, librdf_new_query_from_factory,
    librdf_new_query_from_query, librdf_new_query_results_formatter,
    librdf_new_query_results_formatter_by_mime_type, librdf_new_query_results_formatter2,
    librdf_query_execute, librdf_query_get_limit, librdf_query_get_offset,
    librdf_query_language_get_description, librdf_query_languages_enumerate,
    librdf_query_register_factory, librdf_query_results_as_stream, librdf_query_results_finished,
    librdf_query_results_formats_check, librdf_query_results_formats_enumerate,
    librdf_query_results_formats_get_description, librdf_query_results_formatter_write,
    librdf_query_results_get_binding_name, librdf_query_results_get_binding_value,
    librdf_query_results_get_binding_value_by_name, librdf_query_results_get_bindings,
    librdf_query_results_get_bindings_count, librdf_query_results_get_boolean,
    librdf_query_results_get_count, librdf_query_results_is_bindings,
    librdf_query_results_is_boolean, librdf_query_results_is_graph, librdf_query_results_is_syntax,
    librdf_query_results_next, librdf_query_results_to_counted_string,
    librdf_query_results_to_counted_string2, librdf_query_results_to_file,
    librdf_query_results_to_file_handle, librdf_query_results_to_file_handle2,
    librdf_query_results_to_file2, librdf_query_results_to_string, librdf_query_results_to_string2,
    librdf_query_set_limit, librdf_query_set_offset,
};

pub use handles::serializer::{
    librdf_free_serializer, librdf_new_serializer, librdf_new_serializer_from_factory,
    librdf_serializer_check_name, librdf_serializer_enumerate, librdf_serializer_get_description,
    librdf_serializer_get_feature, librdf_serializer_register_factory,
    librdf_serializer_serialize_model, librdf_serializer_serialize_model_to_counted_string,
    librdf_serializer_serialize_model_to_file, librdf_serializer_serialize_model_to_file_handle,
    librdf_serializer_serialize_model_to_iostream, librdf_serializer_serialize_model_to_string,
    librdf_serializer_serialize_stream_to_counted_string,
    librdf_serializer_serialize_stream_to_file, librdf_serializer_serialize_stream_to_file_handle,
    librdf_serializer_serialize_stream_to_iostream, librdf_serializer_serialize_stream_to_string,
    librdf_serializer_set_error, librdf_serializer_set_feature, librdf_serializer_set_namespace,
    librdf_serializer_set_warning,
};

pub use handles::statement::{
    librdf_free_statement, librdf_new_statement, librdf_new_statement_from_nodes,
    librdf_new_statement_from_statement, librdf_new_statement_from_statement2,
    librdf_statement_clear, librdf_statement_decode, librdf_statement_decode_parts,
    librdf_statement_decode2, librdf_statement_encode, librdf_statement_encode_parts,
    librdf_statement_encode_parts2, librdf_statement_encode2, librdf_statement_equals,
    librdf_statement_get_object, librdf_statement_get_predicate, librdf_statement_get_subject,
    librdf_statement_init, librdf_statement_is_complete, librdf_statement_match,
    librdf_statement_print, librdf_statement_set_object, librdf_statement_set_predicate,
    librdf_statement_set_subject, librdf_statement_to_string, librdf_statement_write,
};

pub use handles::storage::{
    librdf_free_storage, librdf_new_storage, librdf_new_storage_from_factory,
    librdf_new_storage_from_storage, librdf_new_storage_with_options, librdf_storage_add_reference,
    librdf_storage_add_statement, librdf_storage_add_statements, librdf_storage_close,
    librdf_storage_contains_statement, librdf_storage_context_add_statement,
    librdf_storage_context_add_statements, librdf_storage_context_as_stream,
    librdf_storage_context_remove_statement, librdf_storage_context_remove_statements,
    librdf_storage_context_serialise, librdf_storage_enumerate, librdf_storage_find_statements,
    librdf_storage_find_statements_in_context, librdf_storage_find_statements_with_options,
    librdf_storage_get_arcs, librdf_storage_get_arcs_in, librdf_storage_get_arcs_out,
    librdf_storage_get_contexts, librdf_storage_get_feature, librdf_storage_get_instance,
    librdf_storage_get_sources, librdf_storage_get_targets, librdf_storage_get_world,
    librdf_storage_has_arc_in, librdf_storage_has_arc_out, librdf_storage_open,
    librdf_storage_query_execute, librdf_storage_register_factory, librdf_storage_remove_reference,
    librdf_storage_remove_statement, librdf_storage_serialise, librdf_storage_set_feature,
    librdf_storage_set_instance, librdf_storage_size, librdf_storage_supports_query,
    librdf_storage_sync, librdf_storage_transaction_commit, librdf_storage_transaction_get_handle,
    librdf_storage_transaction_rollback, librdf_storage_transaction_start,
    librdf_storage_transaction_start_with_handle,
};

pub use handles::stream::{
    librdf_free_stream, librdf_new_empty_stream, librdf_new_stream,
    librdf_new_stream_from_node_iterator, librdf_stream_add_map, librdf_stream_end,
    librdf_stream_get_context, librdf_stream_get_context2, librdf_stream_get_object,
    librdf_stream_next, librdf_stream_print, librdf_stream_write,
};

pub use handles::uri::{
    librdf_free_uri, librdf_new_uri, librdf_new_uri_from_filename, librdf_new_uri_from_uri,
    librdf_new_uri_from_uri_local_name, librdf_new_uri_normalised_to_base,
    librdf_new_uri_relative_to_base, librdf_new_uri2, librdf_uri_as_counted_string,
    librdf_uri_as_string, librdf_uri_compare, librdf_uri_equals, librdf_uri_is_file_uri,
    librdf_uri_print, librdf_uri_to_counted_string, librdf_uri_to_filename, librdf_uri_to_string,
};

pub use handles::world::{
    librdf_destroy_world, librdf_free_world, librdf_init_world, librdf_log_simple,
    librdf_new_world, librdf_world_get_feature, librdf_world_get_raptor, librdf_world_get_rasqal,
    librdf_world_init_mutex, librdf_world_open, librdf_world_set_digest, librdf_world_set_error,
    librdf_world_set_feature, librdf_world_set_logger, librdf_world_set_raptor,
    librdf_world_set_raptor_init_handler, librdf_world_set_rasqal,
    librdf_world_set_rasqal_init_handler, librdf_world_set_warning,
};

// Opaque handle type aliases
pub use handles::digest::librdf_digest;
pub use handles::hash::librdf_hash;
pub use handles::iterator::librdf_iterator;
pub use handles::list::librdf_list;
pub use handles::log_msg::librdf_log_message;
pub use handles::model::librdf_model;
pub use handles::node::librdf_node;
pub use handles::parser::librdf_parser;
pub use handles::query::librdf_query;
pub use handles::query::librdf_query_results;
pub use handles::query::librdf_query_results_formatter;
pub use handles::serializer::librdf_serializer;
pub use handles::statement::librdf_statement;
pub use handles::storage::librdf_storage;
pub use handles::stream::librdf_stream;
pub use handles::uri::librdf_uri;
pub use handles::io::librdf_iostream;
pub use handles::world::librdf_world;
