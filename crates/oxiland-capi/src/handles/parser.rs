//! `librdf_parser` handle.

use std::io::Cursor;
use std::ptr;

use oxiland::io::{Parser, Syntax};

use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::model::librdf_model;
use crate::handles::uri::librdf_uri;
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_MODEL, TAG_PARSER, TAG_URI, TAG_WORLD, TypedHandle, borrow_handle, box_handle,
    cstr_optional, cstr_required, free_handle,
};

pub type librdf_parser = TypedHandle<ParserInner>;

pub struct ParserInner {
    pub syntax: Syntax,
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
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
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
            Ok(syntax) => box_handle(TAG_PARSER, ParserInner { syntax }),
            Err(error) => {
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
        match rdf_parser.load_into(&model.inner.model, Cursor::new(string.as_bytes())) {
            Ok(_) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}
