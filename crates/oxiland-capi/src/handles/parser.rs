//! `librdf_parser` handle.

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::TAG_STREAM;
use crate::handles::io::{FILE, read_iostream_bytes};
use crate::handles::model::librdf_model;
use crate::handles::node::{NodeInner, librdf_node};
use crate::handles::statement::StatementInner;
use crate::handles::stream::{StreamInner, librdf_stream};
use crate::handles::uri::librdf_uri;
use crate::handles::world::{librdf_world, register_baseline_parser, reject_factory_callback};
use crate::handles::{
    TAG_MODEL, TAG_NODE, TAG_PARSER, TAG_URI, TAG_WORLD, TypedHandle, borrow_handle, box_handle,
    cstr_optional, cstr_required, free_handle,
};
use oxigraph::model::Term;
use oxiland::io::Parser as RdfParser;
use oxiland::io::{Parser, Syntax};
use std::collections::HashMap;
use std::ffi::c_void;
use std::io::Cursor;
use std::os::raw::c_char;
use std::ptr;

pub type librdf_parser = TypedHandle<ParserInner>;

pub struct ParserInner {
    pub syntax: Syntax,
    pub features: HashMap<String, String>,
    pub uri_filter: *mut c_void,
    pub uri_filter_user_data: *mut c_void,
}

fn resolve_syntax(name: Option<&str>, mime: Option<&str>) -> Result<Syntax, String> {
    if let Some(name) = name {
        return Syntax::from_name(name).map_err(|e| e.to_string());
    }
    if let Some(mime) = mime {
        return Syntax::from_media_type(mime).map_err(|e| e.to_string());
    }
    Ok(Syntax::Turtle)
}

/// Creates a parser (`name` e.g. `"turtle"`).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_parser(
    world: *mut librdf_world,
    name: *const std::os::raw::c_char,
    mime_type: *const std::os::raw::c_char,
    _type_uri: *mut librdf_uri,
) -> *mut librdf_parser {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: world is null or a live world handle.
        let Some(_world_handle) = (unsafe { borrow_handle(world, TAG_WORLD) }) else {
            return ptr::null_mut();
        };
        // SAFETY: optional C strings.
        let name = match unsafe { cstr_optional(name, "name") } {
            Ok(v) => v,
            Err(()) => return ptr::null_mut(),
        };
        let mime = match unsafe { cstr_optional(mime_type, "mime_type") } {
            Ok(v) => v,
            Err(()) => return ptr::null_mut(),
        };
        match resolve_syntax(name, mime) {
            Ok(syntax) => box_handle(
                TAG_PARSER,
                ParserInner {
                    syntax,
                    features: HashMap::new(),
                    uri_filter: ptr::null_mut(),
                    uri_filter_user_data: ptr::null_mut(),
                },
            ),
            Err(error) => {
                // ADR-025: unknown / custom factories must fail observably.
                set_last_error(error);
                ptr::null_mut()
            }
        }
    })
}

/// Frees a parser. Null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_parser(parser: *mut librdf_parser) {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: parser is null or a live parser handle.
        unsafe { free_handle(parser, TAG_PARSER) };
    });
}

/// Returns nonzero if `name` is a known parser syntax.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_check_name(
    _world: *mut librdf_world,
    name: *const std::os::raw::c_char,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: name is a C string when non-null.
        let Some(name) = (unsafe { cstr_required(name, "name") }) else {
            return 0;
        };
        match Syntax::from_name(name) {
            Ok(syntax) if syntax.can_parse() => 1,
            _ => 0,
        }
    })
}

/// Parses a UTF-8 string into the model. Returns nonzero on error.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_parse_string_into_model(
    parser: *mut librdf_parser,
    string: *const std::os::raw::c_char,
    base_uri: *mut librdf_uri,
    model: *mut librdf_model,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: handles are null or live.
        let Some(parser) = (unsafe { borrow_handle(parser, TAG_PARSER) }) else {
            return -1;
        };
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        // SAFETY: string is a C string when non-null.
        let Some(string) = (unsafe { cstr_required(string, "string") }) else {
            return -1;
        };
        let mut rdf_parser = Parser::for_syntax(parser.inner.syntax);
        if !base_uri.is_null() {
            // SAFETY: base_uri is a live uri handle when non-null.
            let Some(base) = (unsafe { borrow_handle(base_uri, TAG_URI) }) else {
                return -1;
            };
            rdf_parser = match rdf_parser.base_iri(base.inner.node.as_str()) {
                Ok(p) => p,
                Err(error) => {
                    set_last_error(error.to_string());
                    return -1;
                }
            };
        }
        model.inner.cached_size = None;
        match rdf_parser.load_into(&model.inner.model, Cursor::new(string.as_bytes())) {
            Ok(_) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

/// Parses a counted UTF-8 string into the model.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_parse_counted_string_into_model(
    parser: *mut librdf_parser,
    string: *const u8,
    length: usize,
    base_uri: *mut librdf_uri,
    model: *mut librdf_model,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(parser) = (unsafe { borrow_handle(parser, TAG_PARSER) }) else {
            return -1;
        };
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        if string.is_null() {
            set_last_error("string is null");
            return -1;
        }
        let bytes = unsafe { std::slice::from_raw_parts(string, length) };
        let text = match std::str::from_utf8(bytes) {
            Ok(t) => t,
            Err(_) => {
                set_last_error("string is not valid UTF-8");
                return -1;
            }
        };
        let mut rdf_parser = Parser::for_syntax(parser.inner.syntax);
        if !base_uri.is_null() {
            let Some(base) = (unsafe { borrow_handle(base_uri, TAG_URI) }) else {
                return -1;
            };
            rdf_parser = match rdf_parser.base_iri(base.inner.node.as_str()) {
                Ok(p) => p,
                Err(error) => {
                    set_last_error(error.to_string());
                    return -1;
                }
            };
        }
        model.inner.cached_size = None;
        match rdf_parser.load_into(&model.inner.model, Cursor::new(text.as_bytes())) {
            Ok(_) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_parser_from_factory(
    world: *mut librdf_world,
    factory: *mut c_void,
) -> *mut librdf_parser {
    let _ = factory;
    librdf_new_parser(world, c"turtle".as_ptr(), ptr::null(), ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_enumerate(
    _world: *mut librdf_world,
    counter: u32,
    name: *mut *const c_char,
    label: *mut *const c_char,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let all = Syntax::all();
        let Ok(idx) = usize::try_from(counter) else {
            return 0;
        };
        let Some(syntax) = all.get(idx).copied().filter(|s| s.can_parse()) else {
            // fall through non-parse-only by index into all
            let Some(syntax) = all.get(idx).copied() else {
                return 0;
            };
            let cname: &'static std::ffi::CStr = match syntax.name() {
                "turtle" => c"turtle",
                "ntriples" => c"ntriples",
                "nquads" => c"nquads",
                "trig" => c"trig",
                "rdfxml" => c"rdfxml",
                _ => c"unknown",
            };
            if !name.is_null() {
                unsafe { *name = cname.as_ptr() };
            }
            if !label.is_null() {
                unsafe { *label = cname.as_ptr() };
            }
            return 1;
        };
        let cname: &'static std::ffi::CStr = match syntax.name() {
            "turtle" => c"turtle",
            "ntriples" => c"ntriples",
            "nquads" => c"nquads",
            "trig" => c"trig",
            "rdfxml" => c"rdfxml",
            _ => c"unknown",
        };
        if !name.is_null() {
            unsafe { *name = cname.as_ptr() };
        }
        if !label.is_null() {
            unsafe { *label = cname.as_ptr() };
        }
        1
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_get_description(
    _world: *mut librdf_world,
    counter: u32,
) -> *const c_void {
    // Opaque without Raptor; non-null cookie for known indexes.
    let all = Syntax::all();
    if (counter as usize) < all.len() {
        (counter as usize + 1) as *const c_void
    } else {
        ptr::null()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_guess_name(
    mime_type: *const c_char,
    buffer: *const u8,
    _len: usize,
    identifier: *const c_char,
) -> *const c_char {
    librdf_parser_guess_name2(ptr::null_mut(), mime_type, buffer, identifier)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_guess_name2(
    _world: *mut librdf_world,
    mime_type: *const c_char,
    buffer: *const u8,
    identifier: *const c_char,
) -> *const c_char {
    abort_on_panic(|| {
        clear_last_error();
        if let Ok(Some(mime)) = unsafe { cstr_optional(mime_type, "mime_type") } {
            if let Ok(syntax) = Syntax::from_media_type(mime) {
                return match syntax.name() {
                    "turtle" => c"turtle".as_ptr(),
                    "ntriples" => c"ntriples".as_ptr(),
                    "nquads" => c"nquads".as_ptr(),
                    "trig" => c"trig".as_ptr(),
                    "rdfxml" => c"rdfxml".as_ptr(),
                    _ => ptr::null(),
                };
            }
        }
        if let Ok(Some(id)) = unsafe { cstr_optional(identifier, "identifier") } {
            let lower = id.to_ascii_lowercase();
            if lower.ends_with(".ttl") || lower.ends_with(".turtle") {
                return c"turtle".as_ptr();
            }
            if lower.ends_with(".nt") {
                return c"ntriples".as_ptr();
            }
            if lower.ends_with(".nq") {
                return c"nquads".as_ptr();
            }
            if lower.ends_with(".trig") {
                return c"trig".as_ptr();
            }
            if lower.ends_with(".rdf") || lower.ends_with(".xml") {
                return c"rdfxml".as_ptr();
            }
        }
        if !buffer.is_null() {
            return c"turtle".as_ptr();
        }
        ptr::null()
    })
}

fn parse_bytes_to_stream(
    parser: &ParserInner,
    bytes: &[u8],
    base_uri: *mut librdf_uri,
) -> *mut librdf_stream {
    let mut rdf_parser = RdfParser::for_syntax(parser.syntax);
    if !base_uri.is_null() {
        let Some(base) = (unsafe { borrow_handle(base_uri, TAG_URI) }) else {
            return ptr::null_mut();
        };
        rdf_parser = match rdf_parser.base_iri(base.inner.node.as_str()) {
            Ok(p) => p,
            Err(e) => {
                set_last_error(e.to_string());
                return ptr::null_mut();
            }
        };
    }
    let mut statements = Vec::new();
    // Parse into a temporary model then stream.
    let tmp = match oxiland::Model::new() {
        Ok(m) => m,
        Err(e) => {
            set_last_error(e.to_string());
            return ptr::null_mut();
        }
    };
    if let Err(e) = rdf_parser.load_into(&tmp, Cursor::new(bytes)) {
        set_last_error(e.to_string());
        return ptr::null_mut();
    }
    for item in tmp.find(oxiland::StatementPattern::default()) {
        match item {
            Ok(quad) => {
                statements.push(StatementInner::from_triple(oxigraph::model::Triple::new(
                    quad.subject,
                    quad.predicate,
                    quad.object,
                )));
            }
            Err(e) => {
                set_last_error(e.to_string());
                return ptr::null_mut();
            }
        }
    }
    box_handle(
        TAG_STREAM,
        StreamInner {
            statements,
            triples: Vec::new(),
            index: 0,
            current: None,
        },
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_parse_as_stream(
    parser: *mut librdf_parser,
    uri: *mut librdf_uri,
    base_uri: *mut librdf_uri,
) -> *mut librdf_stream {
    abort_on_panic(|| {
        clear_last_error();
        let Some(parser) = (unsafe { borrow_handle(parser, TAG_PARSER) }) else {
            return ptr::null_mut();
        };
        let Some(uri) = (unsafe { borrow_handle(uri, TAG_URI) }) else {
            return ptr::null_mut();
        };
        let path = uri
            .inner
            .node
            .as_str()
            .strip_prefix("file://")
            .unwrap_or(uri.inner.node.as_str());
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                set_last_error(e.to_string());
                return ptr::null_mut();
            }
        };
        parse_bytes_to_stream(&parser.inner, &bytes, base_uri)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_parse_into_model(
    parser: *mut librdf_parser,
    uri: *mut librdf_uri,
    base_uri: *mut librdf_uri,
    model: *mut librdf_model,
) -> i32 {
    let stream = librdf_parser_parse_as_stream(parser, uri, base_uri);
    if stream.is_null() {
        return -1;
    }
    let rc = crate::handles::model::librdf_model_add_statements(model, stream);
    crate::handles::stream::librdf_free_stream(stream);
    rc
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_parse_string_as_stream(
    parser: *mut librdf_parser,
    string: *const u8,
    base_uri: *mut librdf_uri,
) -> *mut librdf_stream {
    abort_on_panic(|| {
        clear_last_error();
        let Some(parser) = (unsafe { borrow_handle(parser, TAG_PARSER) }) else {
            return ptr::null_mut();
        };
        let Some(string) = (unsafe { cstr_required(string.cast(), "string") }) else {
            return ptr::null_mut();
        };
        parse_bytes_to_stream(&parser.inner, string.as_bytes(), base_uri)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_parse_counted_string_as_stream(
    parser: *mut librdf_parser,
    string: *const u8,
    length: usize,
    base_uri: *mut librdf_uri,
) -> *mut librdf_stream {
    abort_on_panic(|| {
        clear_last_error();
        let Some(parser) = (unsafe { borrow_handle(parser, TAG_PARSER) }) else {
            return ptr::null_mut();
        };
        if string.is_null() {
            set_last_error("string is null");
            return ptr::null_mut();
        }
        let bytes = unsafe { std::slice::from_raw_parts(string, length) };
        parse_bytes_to_stream(&parser.inner, bytes, base_uri)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_parse_file_handle_as_stream(
    parser: *mut librdf_parser,
    fh: *mut FILE,
    _close_fh: i32,
    base_uri: *mut librdf_uri,
) -> *mut librdf_stream {
    abort_on_panic(|| {
        clear_last_error();
        let Some(parser) = (unsafe { borrow_handle(parser, TAG_PARSER) }) else {
            return ptr::null_mut();
        };
        if fh.is_null() {
            set_last_error("file handle is null");
            return ptr::null_mut();
        }
        let mut bytes = Vec::new();
        loop {
            let mut buf = [0u8; 4096];
            let n = unsafe { libc::fread(buf.as_mut_ptr().cast(), 1, buf.len(), fh) };
            if n == 0 {
                if unsafe { libc::ferror(fh) } != 0 {
                    set_last_error("fread I/O error while parsing file handle");
                    return ptr::null_mut();
                }
                break;
            }
            bytes.extend_from_slice(&buf[..n]);
        }
        parse_bytes_to_stream(&parser.inner, &bytes, base_uri)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_parse_file_handle_into_model(
    parser: *mut librdf_parser,
    fh: *mut FILE,
    close_fh: i32,
    base_uri: *mut librdf_uri,
    model: *mut librdf_model,
) -> i32 {
    let stream = librdf_parser_parse_file_handle_as_stream(parser, fh, close_fh, base_uri);
    if stream.is_null() {
        return -1;
    }
    let rc = crate::handles::model::librdf_model_add_statements(model, stream);
    crate::handles::stream::librdf_free_stream(stream);
    if close_fh != 0 && !fh.is_null() {
        unsafe { libc::fclose(fh) };
    }
    rc
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_parse_iostream_as_stream(
    parser: *mut librdf_parser,
    iostream: *mut c_void,
    base_uri: *mut librdf_uri,
) -> *mut librdf_stream {
    abort_on_panic(|| {
        clear_last_error();
        let Some(parser) = (unsafe { borrow_handle(parser, TAG_PARSER) }) else {
            return ptr::null_mut();
        };
        let bytes = match read_iostream_bytes(iostream) {
            Ok(b) => b,
            Err(error) => {
                set_last_error(error);
                return ptr::null_mut();
            }
        };
        parse_bytes_to_stream(&parser.inner, &bytes, base_uri)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_parse_iostream_into_model(
    parser: *mut librdf_parser,
    iostream: *mut c_void,
    base_uri: *mut librdf_uri,
    model: *mut librdf_model,
) -> i32 {
    let stream = librdf_parser_parse_iostream_as_stream(parser, iostream, base_uri);
    if stream.is_null() {
        return -1;
    }
    let rc = crate::handles::model::librdf_model_add_statements(model, stream);
    crate::handles::stream::librdf_free_stream(stream);
    rc
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_get_accept_header(parser: *mut librdf_parser) -> *mut c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(parser) = (unsafe { borrow_handle(parser, TAG_PARSER) }) else {
            return ptr::null_mut();
        };
        strdup_c(parser.inner.syntax.media_type())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_get_namespaces_seen_count(parser: *mut librdf_parser) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(parser, TAG_PARSER) }.is_none() {
            return -1;
        }
        set_last_error("librdf_parser_get_namespaces_seen_* is unsupported");
        -1
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_get_namespaces_seen_prefix(
    parser: *mut librdf_parser,
    _ordinal: i32,
) -> *const c_char {
    abort_on_panic(|| {
        clear_last_error();
        let _ = parser;
        set_last_error("librdf_parser_get_namespaces_seen_* is unsupported");
        ptr::null()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_get_namespaces_seen_uri(
    parser: *mut librdf_parser,
    _ordinal: i32,
) -> *mut librdf_uri {
    abort_on_panic(|| {
        clear_last_error();
        let _ = parser;
        set_last_error("librdf_parser_get_namespaces_seen_* is unsupported");
        ptr::null_mut()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_get_feature(
    parser: *mut librdf_parser,
    feature: *mut librdf_uri,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        let Some(parser) = (unsafe { borrow_handle(parser, TAG_PARSER) }) else {
            return ptr::null_mut();
        };
        let Some(feature) = (unsafe { borrow_handle(feature, TAG_URI) }) else {
            return ptr::null_mut();
        };
        match parser.inner.features.get(feature.inner.node.as_str()) {
            Some(v) => box_handle(
                TAG_NODE,
                NodeInner::from_term(Term::Literal(oxigraph::model::Literal::new_simple_literal(
                    v,
                ))),
            ),
            None => ptr::null_mut(),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_set_feature(
    parser: *mut librdf_parser,
    feature: *mut librdf_uri,
    value: *mut librdf_node,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(parser) = (unsafe { borrow_handle(parser, TAG_PARSER) }) else {
            return -1;
        };
        let Some(feature) = (unsafe { borrow_handle(feature, TAG_URI) }) else {
            return -1;
        };
        let Some(value) = (unsafe { borrow_handle(value, TAG_NODE) }) else {
            return -1;
        };
        let text = match &value.inner.term {
            Term::Literal(l) => l.value().to_owned(),
            other => other.to_string(),
        };
        parser
            .inner
            .features
            .insert(feature.inner.node.as_str().to_owned(), text);
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_set_error(
    parser: *mut librdf_parser,
    _user_data: *mut c_void,
    _error_fn: *mut c_void,
) {
    let _ = parser;
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_set_warning(
    parser: *mut librdf_parser,
    _user_data: *mut c_void,
    _warning_fn: *mut c_void,
) {
    let _ = parser;
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_get_uri_filter(
    parser: *mut librdf_parser,
    user_data_p: *mut *mut c_void,
) -> *mut c_void {
    abort_on_panic(|| {
        clear_last_error();
        let Some(handle) = (unsafe { borrow_handle(parser, TAG_PARSER) }) else {
            return ptr::null_mut();
        };
        if !user_data_p.is_null() {
            unsafe {
                *user_data_p = handle.inner.uri_filter_user_data;
            }
        }
        handle.inner.uri_filter
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_set_uri_filter(
    parser: *mut librdf_parser,
    filter: *mut c_void,
    user_data: *mut c_void,
) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(handle) = (unsafe { borrow_handle(parser, TAG_PARSER) }) else {
            return;
        };
        handle.inner.uri_filter = filter;
        handle.inner.uri_filter_user_data = user_data;
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_parser_register_factory(
    world: *mut librdf_world,
    name: *const c_char,
    _label: *const c_char,
    _mime_type: *const c_char,
    _uri_string: *const u8,
    factory: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(handle) = (unsafe { borrow_handle(world, TAG_WORLD) }) else {
            return;
        };
        let Some(name) = (unsafe { cstr_required(name, "name") }) else {
            return;
        };
        // ADR-025: never execute caller factory callbacks; reject non-baseline names.
        if reject_factory_callback(factory) {
            set_last_error(
                "parser factory callbacks are unsupported; register baseline names only",
            );
            return;
        }
        if let Err(error) = register_baseline_parser(&mut handle.inner, name) {
            set_last_error(error);
        }
    });
}
