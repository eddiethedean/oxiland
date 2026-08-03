//! FFI lifecycle tests calling the C API via the crate's `extern "C"` functions.

use std::ffi::{CStr, CString, c_void};
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
fn model_context_operations_are_isolated_from_default_graph() {
    let world = librdf_new_world();
    let storage = librdf_new_storage(world, cstr("memory").as_ptr(), ptr::null(), ptr::null());
    let model = librdf_new_model(world, storage, ptr::null());
    let context = librdf_new_node_from_uri_string(world, cstr("http://example.org/graph").as_ptr());
    let subject = librdf_new_node_from_uri_string(world, cstr("http://example.org/s").as_ptr());
    let predicate = librdf_new_node_from_uri_string(world, cstr("http://example.org/p").as_ptr());
    let object = librdf_new_node_from_literal(world, cstr("value").as_ptr(), ptr::null(), 0);
    let statement = librdf_new_statement_from_nodes(world, subject, predicate, object);

    assert_eq!(librdf_model_supports_contexts(model), 1);
    assert_eq!(
        librdf_model_context_add_statement(model, context, statement),
        0
    );
    assert_eq!(librdf_model_size(model), 1);
    assert_eq!(librdf_model_contains_context(model, context), 1);
    assert_eq!(librdf_model_contains_statement(model, statement), 0);

    let stream = librdf_model_context_as_stream(model, context);
    assert!(!stream.is_null());
    assert_eq!(librdf_stream_end(stream), 0);
    assert!(!librdf_stream_get_object(stream).is_null());
    librdf_free_stream(stream);

    let pattern = librdf_new_statement(world);
    let matches = librdf_model_find_statements_in_context(model, pattern, context);
    assert!(!matches.is_null());
    assert_eq!(librdf_stream_end(matches), 0);
    librdf_free_stream(matches);
    librdf_free_statement(pattern);

    assert_eq!(
        librdf_model_context_remove_statement(model, context, statement),
        0
    );
    assert_eq!(librdf_model_size(model), 0);
    assert_eq!(librdf_model_contains_context(model, context), 0);
    librdf_free_statement(statement);
    librdf_free_node(context);
    librdf_free_model(model);
    librdf_free_storage(storage);
    librdf_free_world(world);
}

#[test]
fn model_literal_convenience_operations() {
    let world = librdf_new_world();
    let storage = librdf_new_storage(world, cstr("memory").as_ptr(), ptr::null(), ptr::null());
    let model = librdf_new_model(world, storage, ptr::null());
    let subject = librdf_new_node_from_uri_string(world, cstr("http://example.org/s").as_ptr());
    let predicate = librdf_new_node_from_uri_string(world, cstr("http://example.org/p").as_ptr());
    assert_eq!(
        librdf_model_add_string_literal_statement(
            model,
            subject,
            predicate,
            cstr("bonjour").as_ptr().cast(),
            cstr("fr").as_ptr(),
            0,
        ),
        0
    );
    let datatype = librdf_new_uri(
        world,
        cstr("http://www.w3.org/2001/XMLSchema#integer").as_ptr(),
    );
    assert_eq!(
        librdf_model_add_typed_literal_statement(
            model,
            subject,
            predicate,
            cstr("42").as_ptr().cast(),
            ptr::null(),
            datatype,
        ),
        0
    );
    assert_eq!(librdf_model_size(model), 2);
    librdf_free_uri(datatype);
    librdf_free_node(subject);
    librdf_free_node(predicate);
    librdf_free_model(model);
    librdf_free_storage(storage);
    librdf_free_world(world);
}

#[test]
fn statement_clear_and_partial_match_follow_redland_semantics() {
    let world = librdf_new_world();
    let statement = librdf_new_statement_from_nodes(
        world,
        librdf_new_node_from_uri_string(world, cstr("http://example.org/s").as_ptr()),
        librdf_new_node_from_uri_string(world, cstr("http://example.org/p").as_ptr()),
        librdf_new_node_from_literal(world, cstr("o").as_ptr(), ptr::null(), 0),
    );
    let partial = librdf_new_statement_from_nodes(
        world,
        librdf_new_node_from_uri_string(world, cstr("http://example.org/s").as_ptr()),
        ptr::null_mut(),
        ptr::null_mut(),
    );
    assert_eq!(librdf_statement_match(statement, partial), 1);
    librdf_statement_clear(partial);
    assert_eq!(librdf_statement_is_complete(partial), 0);
    assert_eq!(librdf_statement_match(statement, partial), 1);
    librdf_free_statement(partial);
    librdf_free_statement(statement);
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

#[test]
fn freed_and_wrongly_typed_handles_are_rejected_without_dereference() {
    let world = librdf_new_world();
    let node = librdf_new_node_from_uri_string(world, cstr("http://example.org/node").as_ptr());
    assert_eq!(librdf_node_get_type(node), 1);

    // A mismatched free must not unregister or destroy the live node.
    librdf_free_uri(node.cast());
    assert_eq!(librdf_node_get_type(node), 1);

    librdf_free_node(node);
    // A regular API call with the stale pointer is rejected by the registry.
    assert_eq!(librdf_node_get_type(node), 0);
    librdf_free_world(world);
}

#[test]
fn invalid_language_tag_is_rejected_instead_of_changing_literal_kind() {
    let world = librdf_new_world();
    let node = librdf_new_node_from_literal(
        world,
        cstr("hello").as_ptr(),
        cstr("not a language tag!").as_ptr(),
        0,
    );
    assert!(node.is_null());
    librdf_free_world(world);
}

unsafe extern "C" fn clear_logger_from_callback(
    user_data: *mut c_void,
    _code: i32,
    _level: i32,
    _facility: i32,
    _message: *const c_char,
    _locator: *const c_char,
) -> i32 {
    librdf_world_set_logger(user_data.cast(), ptr::null_mut(), None)
}

#[test]
fn logger_callback_can_replace_itself_without_deadlock() {
    let world = librdf_new_world();
    assert_eq!(
        librdf_world_set_logger(world, world.cast(), Some(clear_logger_from_callback),),
        0
    );
    librdf_log_simple(world, 0, 1, 0, ptr::null_mut(), cstr("hello").as_ptr());
    librdf_free_world(world);
}

#[test]
fn hash_list_iterator_lifecycle() {
    let world = librdf_new_world();
    let hash = librdf_new_hash(world, cstr("memory").as_ptr());
    assert!(!hash.is_null());
    assert_eq!(
        librdf_hash_put_strings(hash, cstr("k").as_ptr(), cstr("v").as_ptr()),
        0
    );
    let got = librdf_hash_get(hash, cstr("k").as_ptr());
    assert!(!got.is_null());
    assert_eq!(unsafe { CStr::from_ptr(got) }.to_string_lossy(), "v");
    librdf_free_memory(got.cast());
    librdf_free_hash(hash);

    let list = librdf_new_list(world);
    assert!(!list.is_null());
    let a = std::ptr::dangling_mut::<c_void>();
    let b = 2usize as *mut c_void;
    assert_eq!(librdf_list_add(list, a), 0);
    assert_eq!(librdf_list_unshift(list, b), 0);
    assert_eq!(librdf_list_size(list), 2);
    assert_eq!(librdf_list_contains(list, a), 1);
    assert_eq!(librdf_list_shift(list), b);
    assert_eq!(librdf_list_pop(list), a);
    let it = librdf_list_get_iterator(list);
    assert!(!it.is_null());
    assert_eq!(librdf_iterator_end(it), 1);
    librdf_free_iterator(it);
    librdf_free_list(list);

    let empty = librdf_new_empty_iterator(world);
    assert!(!empty.is_null());
    assert_eq!(librdf_iterator_end(empty), 1);
    librdf_free_iterator(empty);
    librdf_free_list(ptr::null_mut());
    librdf_free_hash(ptr::null_mut());
    librdf_free_iterator(ptr::null_mut());
    librdf_free_world(world);
}

#[test]
fn alloc_and_raptor_bridge_lifecycle() {
    let p = librdf_alloc_memory(16);
    assert!(!p.is_null());
    librdf_free_memory(p);
    let z = librdf_calloc_memory(4, 8);
    assert!(!z.is_null());
    librdf_free_memory(z);

    let world = librdf_new_world();
    let marker = 0xABCDusize as *mut c_void;
    librdf_world_set_raptor(world, marker);
    assert_eq!(librdf_world_get_raptor(world), marker);
    librdf_world_set_rasqal(world, marker);
    assert_eq!(librdf_world_get_rasqal(world), marker);
    assert_eq!(
        librdf_storage_register_factory(world, cstr("memory").as_ptr(), ptr::null(), None),
        0
    );
    assert_eq!(
        librdf_storage_register_factory(world, cstr("nope").as_ptr(), ptr::null(), None),
        -1
    );
    librdf_free_world(world);
}

static mut FACTORY_HITS: u32 = 0;

unsafe extern "C" fn count_factory(_factory: *mut c_void) {
    unsafe { FACTORY_HITS += 1 };
}

#[test]
fn parser_iostream_parse_and_serialize_roundtrip() {
    let world = librdf_new_world();
    librdf_world_open(world);
    let storage = librdf_new_storage(world, cstr("memory").as_ptr(), ptr::null(), ptr::null());
    let model = librdf_new_model(world, storage, ptr::null());
    let parser = librdf_new_parser(world, cstr("turtle").as_ptr(), ptr::null(), ptr::null_mut());
    assert!(!parser.is_null());

    let turtle = b"<http://example.org/s> <http://example.org/p> <http://example.org/o> .\n";
    let iostream = oxiland_new_iostream_from_bytes(turtle.to_vec());
    assert_eq!(
        librdf_parser_parse_iostream_into_model(parser, iostream.cast(), ptr::null_mut(), model),
        0
    );
    assert_eq!(librdf_model_size(model), 1);

    let serializer = librdf_new_serializer(
        world,
        cstr("ntriples").as_ptr(),
        ptr::null(),
        ptr::null_mut(),
    );
    let out = oxiland_new_iostream();
    assert_eq!(
        librdf_serializer_serialize_model_to_iostream(
            serializer,
            ptr::null_mut(),
            model,
            out.cast()
        ),
        0
    );
    let bytes = oxiland_iostream_data(out).expect("iostream bytes");
    assert!(
        std::str::from_utf8(&bytes)
            .unwrap()
            .contains("example.org/s")
    );

    // Unknown non-null iostream must fail (not silent success).
    let garbage = std::ptr::dangling_mut::<c_void>();
    assert_eq!(
        librdf_serializer_serialize_model_to_iostream(serializer, ptr::null_mut(), model, garbage),
        -1
    );
    assert!(
        librdf_parser_parse_iostream_as_stream(parser, ptr::null_mut(), ptr::null_mut()).is_null()
    );

    oxiland_free_iostream(iostream);
    oxiland_free_iostream(out);
    librdf_free_serializer(serializer);
    librdf_free_parser(parser);
    librdf_free_model(model);
    librdf_free_storage(storage);
    librdf_free_world(world);
}

#[test]
fn parser_factory_rejects_callback_and_unknown_name() {
    unsafe { FACTORY_HITS = 0 };
    let world = librdf_new_world();
    librdf_parser_register_factory(
        world,
        cstr("custom-parser").as_ptr(),
        ptr::null(),
        ptr::null(),
        ptr::null(),
        Some(count_factory),
    );
    // ADR-025: callbacks are not executed.
    assert_eq!(unsafe { FACTORY_HITS }, 0);

    let parser = librdf_new_parser(
        world,
        cstr("custom-parser").as_ptr(),
        ptr::null(),
        ptr::null_mut(),
    );
    assert!(parser.is_null());

    // Baseline name without callback succeeds; features still round-trip.
    librdf_parser_register_factory(
        world,
        cstr("turtle").as_ptr(),
        ptr::null(),
        ptr::null(),
        ptr::null(),
        None,
    );
    let parser = librdf_new_parser(world, cstr("turtle").as_ptr(), ptr::null(), ptr::null_mut());
    assert!(!parser.is_null());
    let feature_uri = librdf_new_uri(world, cstr("http://example.org/feature").as_ptr());
    let value = librdf_new_node_from_literal(world, cstr("on").as_ptr(), ptr::null(), 0);
    assert_eq!(librdf_parser_set_feature(parser, feature_uri, value), 0);
    let got = librdf_parser_get_feature(parser, feature_uri);
    assert!(!got.is_null());
    let lit = librdf_node_get_literal_value(got);
    assert_eq!(unsafe { CStr::from_ptr(lit) }.to_string_lossy(), "on");
    librdf_free_node(got);
    librdf_free_node(value);
    librdf_free_uri(feature_uri);
    librdf_free_parser(parser);
    librdf_free_world(world);
}
