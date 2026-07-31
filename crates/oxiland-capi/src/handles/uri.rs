//! `librdf_uri` handle.

use std::ptr;

use oxigraph::model::NamedNode;

use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_URI, TAG_WORLD, TypedHandle, borrow_handle, box_handle, cstr_required, free_handle,
};

pub type librdf_uri = TypedHandle<UriInner>;

pub struct UriInner {
    pub node: NamedNode,
}

/// Creates a URI from a string.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_uri(
    world: *mut librdf_world,
    uri_string: *const std::os::raw::c_char,
) -> *mut librdf_uri {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: world is null or a live world handle.
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        // SAFETY: uri_string is a C string when non-null.
        let Some(uri_string) = (unsafe { cstr_required(uri_string, "uri_string") }) else {
            return ptr::null_mut();
        };
        match NamedNode::new(uri_string) {
            Ok(node) => box_handle(TAG_URI, UriInner { node }),
            Err(error) => {
                set_last_error(error.to_string());
                ptr::null_mut()
            }
        }
    })
}

/// Frees a URI. Null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_uri(uri: *mut librdf_uri) {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: uri is null or a live uri handle.
        unsafe { free_handle(uri, TAG_URI) };
    });
}
