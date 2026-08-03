//! Parsing, serialization, and stream I/O for the C model adapter.

use super::{librdf_model, librdf_model_as_stream};
use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::io::{FILE, write_iostream, writeln_file};
use crate::handles::stream::librdf_stream;
use crate::handles::uri::librdf_uri;
use crate::handles::{TAG_MODEL, TAG_URI, borrow_handle, cstr_optional};
use oxiland::io::{Parser, Serializer, Syntax};
use std::ffi::c_void;
use std::io::Cursor;
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;

fn syntax_from_hints(name: *const c_char, mime_type: *const c_char) -> Syntax {
    let name = unsafe { cstr_optional(name, "name") }.ok().flatten();
    let mime_type = unsafe { cstr_optional(mime_type, "mime_type") }
        .ok()
        .flatten();
    if let Some(name) = name {
        Syntax::from_name(name).unwrap_or(Syntax::Turtle)
    } else if let Some(mime_type) = mime_type {
        Syntax::from_media_type(mime_type).unwrap_or(Syntax::Turtle)
    } else {
        Syntax::Turtle
    }
}

/// Serializes the model as Turtle to a malloc'd string.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_to_string(
    model: *mut librdf_model,
    _base_uri: *mut crate::handles::uri::librdf_uri,
) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        match Serializer::for_syntax(Syntax::Turtle).serialize_model_to_string(&model.inner.model) {
            Ok(text) => crate::alloc::strdup_c(&text).cast(),
            Err(error) => {
                set_last_error(error.to_string());
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_load(
    model: *mut librdf_model,
    uri: *mut librdf_uri,
    name: *const c_char,
    mime_type: *const c_char,
    _type_uri: *mut librdf_uri,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        let Some(uri) = (unsafe { borrow_handle(uri, TAG_URI) }) else {
            return -1;
        };
        let syntax = syntax_from_hints(name, mime_type);
        let path = match oxiland::utility::file_uri_to_path(uri.inner.node.as_str()) {
            Ok(p) => p,
            Err(_) => {
                // Treat as local path string
                Path::new(uri.inner.node.as_str()).to_path_buf()
            }
        };
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                set_last_error(e.to_string());
                return -1;
            }
        };
        model.inner.cardinality.invalidate();
        match Parser::for_syntax(syntax).load_into(&model.inner.model, Cursor::new(bytes)) {
            Ok(_) => 0,
            Err(e) => {
                set_last_error(e.to_string());
                -1
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_to_counted_string(
    model: *mut librdf_model,
    _uri: *mut librdf_uri,
    name: *const c_char,
    mime_type: *const c_char,
    _type_uri: *mut librdf_uri,
    string_length_p: *mut usize,
) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let syntax = syntax_from_hints(name, mime_type);
        match Serializer::for_syntax(syntax).serialize_model_to_string(&model.inner.model) {
            Ok(text) => {
                if !string_length_p.is_null() {
                    unsafe { *string_length_p = text.len() };
                }
                strdup_c(&text).cast()
            }
            Err(e) => {
                set_last_error(e.to_string());
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_print(model: *mut librdf_model, fh: *mut FILE) {
    abort_on_panic(|| {
        clear_last_error();
        let text_ptr = librdf_model_to_string(model, ptr::null_mut());
        if text_ptr.is_null() {
            return;
        }
        let text = unsafe { std::ffi::CStr::from_ptr(text_ptr.cast()) }.to_string_lossy();
        let _ = writeln_file(fh, &text);
        crate::alloc::librdf_free_memory(text_ptr.cast());
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_write(model: *mut librdf_model, iostr: *mut c_void) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let text_ptr = librdf_model_to_string(model, ptr::null_mut());
        if text_ptr.is_null() {
            return -1;
        }
        let text = unsafe { std::ffi::CStr::from_ptr(text_ptr.cast()) }.to_bytes();
        let rc = write_iostream(iostr, text);
        crate::alloc::librdf_free_memory(text_ptr.cast());
        rc
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_serialise(model: *mut librdf_model) -> *mut librdf_stream {
    librdf_model_as_stream(model)
}
