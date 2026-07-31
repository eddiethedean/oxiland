//! C ABI preview for Oxiland (Redland-shaped source-compat).
//!
//! All `unsafe` lives here; the safe `oxiland` crate forbids it (ADR-002).
//! Ownership, panic containment, and allocator rules: ADR-023 /
//! `docs/design/0.8-cabi.md`.

#![allow(non_camel_case_types)]
#![allow(missing_docs)]
// extern "C" entry points intentionally take raw pointers; marking them `unsafe
// fn` would break the C calling convention for Redland-shaped callers.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod alloc;
mod error;
mod handles;

pub use alloc::librdf_free_memory;
pub use handles::digest::{
    librdf_digest_final, librdf_digest_get_digest, librdf_digest_get_digest_length,
    librdf_digest_init, librdf_digest_to_string, librdf_digest_update, librdf_digest_update_string,
    librdf_free_digest, librdf_new_digest,
};
pub use handles::helpers::{librdf_basename, librdf_latin1_to_utf8, librdf_utf8_to_latin1};
pub use handles::model::{
    librdf_free_model, librdf_model_add, librdf_model_add_statement,
    librdf_model_add_string_literal_statement, librdf_model_add_typed_literal_statement,
    librdf_model_as_stream, librdf_model_contains_context, librdf_model_contains_statement,
    librdf_model_context_add_statement, librdf_model_context_as_stream,
    librdf_model_context_remove_statement, librdf_model_find_statements,
    librdf_model_find_statements_in_context, librdf_model_remove_statement, librdf_model_size,
    librdf_model_supports_contexts, librdf_model_sync, librdf_model_to_string, librdf_model_update,
    librdf_new_model,
};
pub use handles::node::{
    librdf_free_node, librdf_new_node_from_blank_identifier, librdf_new_node_from_literal,
    librdf_new_node_from_uri_string, librdf_node_equals, librdf_node_get_blank_identifier,
    librdf_node_get_literal_value, librdf_node_get_literal_value_language, librdf_node_get_type,
    librdf_node_get_uri, librdf_node_is_blank, librdf_node_is_literal, librdf_node_is_resource,
    librdf_node_to_string,
};
pub use handles::parser::{
    librdf_free_parser, librdf_new_parser, librdf_parser_check_name,
    librdf_parser_parse_counted_string_into_model, librdf_parser_parse_string_into_model,
};
pub use handles::query::{
    librdf_free_query, librdf_free_query_results, librdf_model_query_execute, librdf_new_query,
    librdf_query_results_as_stream, librdf_query_results_finished,
    librdf_query_results_get_binding_name, librdf_query_results_get_binding_value,
    librdf_query_results_get_bindings_count, librdf_query_results_get_boolean,
    librdf_query_results_is_bindings, librdf_query_results_is_boolean,
    librdf_query_results_is_graph, librdf_query_results_next,
};
pub use handles::serializer::{
    librdf_free_serializer, librdf_new_serializer, librdf_serializer_check_name,
    librdf_serializer_serialize_model_to_counted_string, librdf_serializer_serialize_model_to_file,
    librdf_serializer_serialize_model_to_string,
};
pub use handles::statement::{
    librdf_free_statement, librdf_new_statement, librdf_new_statement_from_nodes,
    librdf_statement_clear, librdf_statement_equals, librdf_statement_get_object,
    librdf_statement_get_predicate, librdf_statement_get_subject, librdf_statement_is_complete,
    librdf_statement_match, librdf_statement_set_object, librdf_statement_set_predicate,
    librdf_statement_set_subject, librdf_statement_to_string,
};
pub use handles::storage::{
    librdf_free_storage, librdf_new_storage, librdf_storage_enumerate, librdf_storage_open,
    librdf_storage_sync,
};
pub use handles::stream::{
    librdf_free_stream, librdf_stream_end, librdf_stream_get_object, librdf_stream_next,
};
pub use handles::uri::{
    librdf_free_uri, librdf_new_uri, librdf_uri_as_string, librdf_uri_compare, librdf_uri_equals,
    librdf_uri_is_file_uri, librdf_uri_to_filename, librdf_uri_to_string,
};
pub use handles::world::{
    librdf_free_world, librdf_log_simple, librdf_new_world, librdf_world_open,
    librdf_world_set_logger,
};

pub use handles::digest::librdf_digest;
pub use handles::model::librdf_model;
pub use handles::node::librdf_node;
pub use handles::parser::librdf_parser;
pub use handles::query::{librdf_query, librdf_query_results};
pub use handles::serializer::librdf_serializer;
pub use handles::statement::librdf_statement;
pub use handles::storage::librdf_storage;
pub use handles::stream::librdf_stream;
pub use handles::uri::librdf_uri;
pub use handles::world::librdf_world;
