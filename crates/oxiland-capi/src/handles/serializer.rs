//! `librdf_serializer` handle.

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::io::{FILE, write_file};
use crate::handles::model::librdf_model;
use crate::handles::node::librdf_node;
use crate::handles::stream::{
    librdf_stream, librdf_stream_end, librdf_stream_get_object, librdf_stream_next,
};
use crate::handles::uri::librdf_uri;
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_MODEL, TAG_SERIALIZER, TAG_URI, TAG_WORLD, TypedHandle, borrow_handle, box_handle,
    cstr_optional, cstr_required, free_handle,
};
use oxiland::io::{Serializer, Syntax};
use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr;

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

/// Serializes the model and returns length via `length_p` when non-null.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_serialize_model_to_counted_string(
    serializer: *mut librdf_serializer,
    base_uri: *mut librdf_uri,
    model: *mut librdf_model,
    length_p: *mut usize,
) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        let ptr = librdf_serializer_serialize_model_to_string(serializer, base_uri, model);
        if ptr.is_null() {
            return ptr::null_mut();
        }
        if !length_p.is_null() {
            let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
            unsafe { *length_p = cstr.to_bytes().len() };
        }
        ptr.cast()
    })
}

/// Serializes the model to a filesystem path.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_serialize_model_to_file(
    serializer: *mut librdf_serializer,
    name: *const std::os::raw::c_char,
    base_uri: *mut librdf_uri,
    model: *mut librdf_model,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(serializer) = (unsafe { borrow_handle(serializer, TAG_SERIALIZER) }) else {
            return -1;
        };
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        let Some(name) = (unsafe { cstr_required(name, "name") }) else {
            return -1;
        };
        let mut ser = Serializer::for_syntax(serializer.inner.syntax);
        if !base_uri.is_null() {
            let Some(base) = (unsafe { borrow_handle(base_uri, TAG_URI) }) else {
                return -1;
            };
            ser = match ser.base_iri(base.inner.node.as_str()) {
                Ok(s) => s,
                Err(error) => {
                    set_last_error(error.to_string());
                    return -1;
                }
            };
        }
        match ser.serialize_model_to_path(&model.inner.model, name) {
            Ok(()) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_serializer_from_factory(
    world: *mut librdf_world,
    factory: *mut c_void,
) -> *mut librdf_serializer {
    let _ = factory;
    librdf_new_serializer(world, c"turtle".as_ptr(), ptr::null(), ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_enumerate(
    _world: *mut librdf_world,
    counter: u32,
    name: *mut *const c_char,
    label: *mut *const c_char,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let all: Vec<_> = Syntax::all()
            .iter()
            .copied()
            .filter(|s| s.can_serialize())
            .collect();
        let Ok(idx) = usize::try_from(counter) else {
            return 0;
        };
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
        1
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_get_description(
    _world: *mut librdf_world,
    counter: u32,
) -> *const c_void {
    if (counter as usize) < Syntax::all().len() {
        (counter as usize + 1) as *const c_void
    } else {
        ptr::null()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_serialize_model(
    serializer: *mut librdf_serializer,
    fh: *mut FILE,
    base_uri: *mut librdf_uri,
    model: *mut librdf_model,
) -> i32 {
    librdf_serializer_serialize_model_to_file_handle(serializer, fh, base_uri, model)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_serialize_model_to_file_handle(
    serializer: *mut librdf_serializer,
    fh: *mut FILE,
    base_uri: *mut librdf_uri,
    model: *mut librdf_model,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let text = librdf_serializer_serialize_model_to_string(serializer, base_uri, model);
        if text.is_null() {
            return -1;
        }
        let bytes = unsafe { std::ffi::CStr::from_ptr(text.cast()) }.to_bytes();
        let rc = write_file(fh, bytes);
        crate::alloc::librdf_free_memory(text.cast());
        rc
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_serialize_model_to_iostream(
    serializer: *mut librdf_serializer,
    base_uri: *mut librdf_uri,
    model: *mut librdf_model,
    iostr: *mut c_void,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let text = librdf_serializer_serialize_model_to_string(serializer, base_uri, model);
        if text.is_null() {
            return -1;
        }
        let bytes = unsafe { std::ffi::CStr::from_ptr(text.cast()) }.to_bytes();
        let rc = crate::handles::io::write_iostream(iostr, bytes);
        crate::alloc::librdf_free_memory(text.cast());
        rc
    })
}

fn stream_to_ntriples(stream: *mut librdf_stream) -> Result<String, String> {
    let mut out = String::new();
    // Rewind not supported; consume from current position.
    while librdf_stream_end(stream) == 0 {
        let stmt = librdf_stream_get_object(stream);
        let p = crate::handles::statement::librdf_statement_to_string(stmt);
        if p.is_null() {
            return Err("statement_to_string failed".into());
        }
        let text = unsafe { std::ffi::CStr::from_ptr(p.cast()) }.to_string_lossy();
        out.push_str(&text);
        if !text.ends_with('\n') {
            out.push('\n');
        }
        crate::alloc::librdf_free_memory(p.cast());
        if librdf_stream_next(stream) != 0 {
            break;
        }
    }
    Ok(out)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_serialize_stream_to_string(
    serializer: *mut librdf_serializer,
    _base_uri: *mut librdf_uri,
    stream: *mut librdf_stream,
) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(serializer, TAG_SERIALIZER) }.is_none() {
            return ptr::null_mut();
        }
        match stream_to_ntriples(stream) {
            Ok(text) => strdup_c(&text).cast(),
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_serialize_stream_to_counted_string(
    serializer: *mut librdf_serializer,
    base_uri: *mut librdf_uri,
    stream: *mut librdf_stream,
    length_p: *mut usize,
) -> *mut u8 {
    let p = librdf_serializer_serialize_stream_to_string(serializer, base_uri, stream);
    if !p.is_null() && !length_p.is_null() {
        let s = unsafe { std::ffi::CStr::from_ptr(p.cast()) };
        unsafe { *length_p = s.to_bytes().len() };
    }
    p
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_serialize_stream_to_file(
    serializer: *mut librdf_serializer,
    name: *const c_char,
    base_uri: *mut librdf_uri,
    stream: *mut librdf_stream,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(name) = (unsafe { cstr_required(name, "name") }) else {
            return -1;
        };
        let text = librdf_serializer_serialize_stream_to_string(serializer, base_uri, stream);
        if text.is_null() {
            return -1;
        }
        let bytes = unsafe { std::ffi::CStr::from_ptr(text.cast()) }.to_bytes();
        let rc = match std::fs::write(name, bytes) {
            Ok(()) => 0,
            Err(e) => {
                set_last_error(e.to_string());
                -1
            }
        };
        crate::alloc::librdf_free_memory(text.cast());
        rc
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_serialize_stream_to_file_handle(
    serializer: *mut librdf_serializer,
    fh: *mut FILE,
    base_uri: *mut librdf_uri,
    stream: *mut librdf_stream,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let text = librdf_serializer_serialize_stream_to_string(serializer, base_uri, stream);
        if text.is_null() {
            return -1;
        }
        let bytes = unsafe { std::ffi::CStr::from_ptr(text.cast()) }.to_bytes();
        let rc = write_file(fh, bytes);
        crate::alloc::librdf_free_memory(text.cast());
        rc
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_serialize_stream_to_iostream(
    serializer: *mut librdf_serializer,
    base_uri: *mut librdf_uri,
    stream: *mut librdf_stream,
    iostr: *mut c_void,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let text = librdf_serializer_serialize_stream_to_string(serializer, base_uri, stream);
        if text.is_null() {
            return -1;
        }
        let bytes = unsafe { std::ffi::CStr::from_ptr(text.cast()) }.to_bytes();
        let rc = crate::handles::io::write_iostream(iostr, bytes);
        crate::alloc::librdf_free_memory(text.cast());
        rc
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_get_feature(
    serializer: *mut librdf_serializer,
    _feature: *mut librdf_uri,
) -> *mut librdf_node {
    let _ = serializer;
    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_set_feature(
    serializer: *mut librdf_serializer,
    _feature: *mut librdf_uri,
    _value: *mut librdf_node,
) -> i32 {
    if unsafe { borrow_handle(serializer, TAG_SERIALIZER) }.is_none() {
        -1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_set_namespace(
    serializer: *mut librdf_serializer,
    _uri: *mut librdf_uri,
    _prefix: *const c_char,
) -> i32 {
    if unsafe { borrow_handle(serializer, TAG_SERIALIZER) }.is_none() {
        -1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_set_error(
    serializer: *mut librdf_serializer,
    _user_data: *mut c_void,
    _error_fn: *mut c_void,
) {
    let _ = serializer;
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_set_warning(
    serializer: *mut librdf_serializer,
    _user_data: *mut c_void,
    _warning_fn: *mut c_void,
) {
    let _ = serializer;
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_serializer_register_factory(
    world: *mut librdf_world,
    name: *const c_char,
    _label: *const c_char,
    _mime_type: *const c_char,
    _uri_string: *const u8,
    _factory: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(handle) = (unsafe { borrow_handle(world, TAG_WORLD) }) else {
            return;
        };
        let Some(name) = (unsafe { cstr_required(name, "name") }) else {
            return;
        };
        if Syntax::from_name(name).is_ok()
            || ["turtle", "ntriples", "nquads", "trig", "rdfxml", "raptor"]
                .iter()
                .any(|k| k.eq_ignore_ascii_case(name))
        {
            handle.inner.registered_serializers.push(name.to_owned());
        } else {
            set_last_error(format!("unknown serializer factory '{name}'"));
        }
    });
}
