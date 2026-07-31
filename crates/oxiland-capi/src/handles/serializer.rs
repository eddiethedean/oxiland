//! `librdf_serializer` handle.

use std::ptr;

use oxiland::io::{Serializer, Syntax};

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::model::librdf_model;
use crate::handles::uri::librdf_uri;
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_MODEL, TAG_SERIALIZER, TAG_URI, TAG_WORLD, TypedHandle, borrow_handle, box_handle,
    cstr_optional, cstr_required, free_handle,
};

pub type librdf_serializer = TypedHandle<SerializerInner>;

pub struct SerializerInner {
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

/// Creates a serializer (`name` e.g. `"turtle"`).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_serializer(
    world: *mut librdf_world,
    name: *const std::os::raw::c_char,
    mime_type: *const std::os::raw::c_char,
    _type_uri: *mut librdf_uri,
) -> *mut librdf_serializer {
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
            Ok(syntax) => box_handle(TAG_SERIALIZER, SerializerInner { syntax }),
            Err(error) => {
                set_last_error(error);
                ptr::null_mut()
            }
        }
    })
}

/// Frees a serializer. Null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_serializer(serializer: *mut librdf_serializer) {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: serializer is null or a live serializer handle.
        unsafe { free_handle(serializer, TAG_SERIALIZER) };
    });
}

/// Returns nonzero if `name` is a known serializer syntax.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_check_name(
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
            Ok(syntax) if syntax.can_serialize() => 1,
            _ => 0,
        }
    })
}

/// Serializes the model to a newly allocated C string (free with `librdf_free_memory`).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_serialize_model_to_string(
    serializer: *mut librdf_serializer,
    base_uri: *mut librdf_uri,
    model: *mut librdf_model,
) -> *mut std::os::raw::c_char {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: handles are null or live.
        let Some(serializer) = (unsafe { borrow_handle(serializer, TAG_SERIALIZER) }) else {
            return ptr::null_mut();
        };
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let mut ser = Serializer::for_syntax(serializer.inner.syntax);
        if !base_uri.is_null() {
            // SAFETY: base_uri is a live uri handle when non-null.
            let Some(base) = (unsafe { borrow_handle(base_uri, TAG_URI) }) else {
                return ptr::null_mut();
            };
            ser = match ser.base_iri(base.inner.node.as_str()) {
                Ok(s) => s,
                Err(error) => {
                    set_last_error(error.to_string());
                    return ptr::null_mut();
                }
            };
        }
        match ser.serialize_model_to_string(&model.inner.model) {
            Ok(text) => strdup_c(&text),
            Err(error) => {
                set_last_error(error.to_string());
                ptr::null_mut()
            }
        }
    })
}
