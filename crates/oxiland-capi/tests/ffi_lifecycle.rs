//! FFI lifecycle tests calling the C API via the crate's `extern "C"` functions.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use oxiland_capi::*;

fn cstr(s: &str) -> CString {
    CString::new(s).expect("no interior NUL")
}

#[test]
fn world_new_free_and_null_free() {
    let world = librdf_new_world();
    assert!(!world.is_null());
    librdf_world_open(world);
    librdf_free_world(world);
    librdf_free_world(ptr::null_mut());
}

#[test]
fn model_add_contains_size() {
    let world = librdf_new_world();
    librdf_world_open(world);
    let storage = librdf_new_storage(world, cstr("memory").as_ptr(), ptr::null(), ptr::null());
    assert!(!storage.is_null());
    let model = librdf_new_model(world, storage, ptr::null());
    assert!(!model.is_null());

    let s = librdf_new_node_from_uri_string(world, cstr("http://example.org/s").as_ptr());
    let p = librdf_new_node_from_uri_string(world, cstr("http://example.org/p").as_ptr());
    let o = librdf_new_node_from_literal(world, cstr("o").as_ptr(), ptr::null(), 0);
    let stmt = librdf_new_statement_from_nodes(world, s, p, o);
    assert!(!stmt.is_null());

    assert_eq!(librdf_model_add_statement(model, stmt), 0);
    assert_eq!(librdf_model_contains_statement(model, stmt), 1);
    assert_eq!(librdf_model_size(model), 1);

    librdf_free_statement(stmt);
    librdf_free_model(model);
    librdf_free_storage(storage);
    librdf_free_world(world);
}

#[test]
fn invalid_utf8_rejected() {
    let world = librdf_new_world();
    let bad: [u8; 3] = [0xff, 0xfe, 0x00];
    let node = librdf_new_node_from_uri_string(world, bad.as_ptr().cast::<c_char>());
    assert!(node.is_null());
    librdf_free_world(world);
}

#[test]
fn null_frees_are_noop() {
    librdf_free_uri(ptr::null_mut());
    librdf_free_node(ptr::null_mut());
    librdf_free_statement(ptr::null_mut());
    librdf_free_stream(ptr::null_mut());
    librdf_free_parser(ptr::null_mut());
    librdf_free_serializer(ptr::null_mut());
    librdf_free_query(ptr::null_mut());
    librdf_free_query_results(ptr::null_mut());
    librdf_free_model(ptr::null_mut());
    librdf_free_storage(ptr::null_mut());
    librdf_free_memory(ptr::null_mut());
}

#[test]
fn parse_turtle_and_ask_query() {
    let world = librdf_new_world();
    librdf_world_open(world);
    let storage = librdf_new_storage(world, cstr("memory").as_ptr(), ptr::null(), ptr::null());
    let model = librdf_new_model(world, storage, ptr::null());

    assert_eq!(librdf_parser_check_name(world, cstr("turtle").as_ptr()), 1);
    let parser = librdf_new_parser(world, cstr("turtle").as_ptr(), ptr::null(), ptr::null_mut());
    assert!(!parser.is_null());
    let turtle = cstr("<http://example.org/a> <http://example.org/b> \"c\" .");
    assert_eq!(
        librdf_parser_parse_string_into_model(parser, turtle.as_ptr(), ptr::null_mut(), model),
        0
    );
    assert!(librdf_model_size(model) >= 1);

    let query = librdf_new_query(
        world,
        cstr("sparql").as_ptr(),
        ptr::null_mut(),
        cstr("ASK { ?s ?p ?o }").as_ptr(),
        ptr::null_mut(),
    );
    assert!(!query.is_null());
    let results = librdf_model_query_execute(model, query);
    assert!(!results.is_null());
    assert_eq!(librdf_query_results_is_boolean(results), 1);
    assert_eq!(librdf_query_results_get_boolean(results), 1);

    librdf_free_query_results(results);
    librdf_free_query(query);
    librdf_free_parser(parser);
    librdf_free_model(model);
    librdf_free_storage(storage);
    librdf_free_world(world);
}

#[test]
fn serialize_and_select() {
    let world = librdf_new_world();
    let storage = librdf_new_storage(world, cstr("memory").as_ptr(), ptr::null(), ptr::null());
    let model = librdf_new_model(world, storage, ptr::null());

    let s = librdf_new_node_from_uri_string(world, cstr("http://example.org/s").as_ptr());
    let p = librdf_new_node_from_uri_string(world, cstr("http://example.org/p").as_ptr());
    let o = librdf_new_node_from_literal(world, cstr("hello").as_ptr(), ptr::null(), 0);
    let stmt = librdf_new_statement_from_nodes(world, s, p, o);
    assert_eq!(librdf_model_add_statement(model, stmt), 0);
    librdf_free_statement(stmt);

    assert_eq!(
        librdf_serializer_check_name(world, cstr("turtle").as_ptr()),
        1
    );
    let ser = librdf_new_serializer(world, cstr("turtle").as_ptr(), ptr::null(), ptr::null_mut());
    let out = librdf_serializer_serialize_model_to_string(ser, ptr::null_mut(), model);
    assert!(!out.is_null());
    // SAFETY: out is a NUL-terminated string from strdup_c.
    let text = unsafe { CStr::from_ptr(out) }.to_string_lossy();
    assert!(text.contains("hello"));
    librdf_free_memory(out.cast());

    let query = librdf_new_query(
        world,
        cstr("sparql").as_ptr(),
        ptr::null_mut(),
        cstr("SELECT ?o WHERE { <http://example.org/s> <http://example.org/p> ?o }").as_ptr(),
        ptr::null_mut(),
    );
    let results = librdf_model_query_execute(model, query);
    assert_eq!(librdf_query_results_is_bindings(results), 1);
    assert_eq!(librdf_query_results_finished(results), 0);
    assert_eq!(librdf_query_results_get_bindings_count(results), 1);
    assert!(!librdf_query_results_get_binding_name(results, 0).is_null());
    assert!(!librdf_query_results_get_binding_value(results, 0).is_null());

    librdf_free_query_results(results);
    librdf_free_query(query);
    librdf_free_serializer(ser);
    librdf_free_model(model);
    librdf_free_storage(storage);
    librdf_free_world(world);
}

#[test]
fn find_stream_lifecycle() {
    let world = librdf_new_world();
    let storage = librdf_new_storage(world, cstr("memory").as_ptr(), ptr::null(), ptr::null());
    let model = librdf_new_model(world, storage, ptr::null());

    let s = librdf_new_node_from_uri_string(world, cstr("http://example.org/s").as_ptr());
    let p = librdf_new_node_from_uri_string(world, cstr("http://example.org/p").as_ptr());
    let o = librdf_new_node_from_literal(world, cstr("o").as_ptr(), ptr::null(), 0);
    let stmt = librdf_new_statement_from_nodes(world, s, p, o);
    assert_eq!(librdf_model_add_statement(model, stmt), 0);
    librdf_free_statement(stmt);

    let pattern =
        librdf_new_statement_from_nodes(world, ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
    let stream = librdf_model_find_statements(model, pattern);
    assert!(!stream.is_null());
    assert_eq!(librdf_stream_end(stream), 0);
    assert!(!librdf_stream_get_object(stream).is_null());
    let _ = librdf_stream_next(stream);

    librdf_free_stream(stream);
    librdf_free_statement(pattern);
    librdf_free_model(model);
    librdf_free_storage(storage);
    librdf_free_world(world);
}

#[test]
fn double_free_is_defended() {
    let world = librdf_new_world();
    assert!(!world.is_null());
    librdf_free_world(world);
    // Second free of the same address: registry rejects without double-drop.
    librdf_free_world(world);
}
