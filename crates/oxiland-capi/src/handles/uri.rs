//! `librdf_uri` handle.

use std::os::raw::c_char;
use std::ptr;

use oxigraph::model::NamedNode;
use oxiland::utility::file_uri_to_path;

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_URI, TAG_WORLD, TypedHandle, borrow_handle, box_handle, cstr_required, free_handle,
};

pub type librdf_uri = TypedHandle<UriInner>;

pub struct UriInner {
    pub node: NamedNode,
    pub as_string: Option<*mut c_char>,
}

impl Drop for UriInner {
    fn drop(&mut self) {
        if let Some(ptr) = self.as_string.take() {
            if !ptr.is_null() {
                unsafe { libc::free(ptr.cast()) };
            }
        }
    }
}

impl UriInner {
    pub fn new(node: NamedNode) -> Self {
        Self {
            node,
            as_string: None,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_uri(
    world: *mut librdf_world,
    uri_string: *const c_char,
) -> *mut librdf_uri {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        let Some(uri_string) = (unsafe { cstr_required(uri_string, "uri_string") }) else {
            return ptr::null_mut();
        };
        match NamedNode::new(uri_string) {
            Ok(node) => box_handle(TAG_URI, UriInner::new(node)),
            Err(error) => {
                set_last_error(error.to_string());
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_uri(uri: *mut librdf_uri) {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { free_handle(uri, TAG_URI) };
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_uri_as_string(uri: *mut librdf_uri) -> *const c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(uri) = (unsafe { borrow_handle(uri, TAG_URI) }) else {
            return ptr::null();
        };
        if uri.inner.as_string.is_none() {
            uri.inner.as_string = Some(strdup_c(uri.inner.node.as_str()));
        }
        uri.inner.as_string.unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_uri_to_string(uri: *mut librdf_uri) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(uri) = (unsafe { borrow_handle(uri, TAG_URI) }) else {
            return ptr::null_mut();
        };
        strdup_c(uri.inner.node.as_str()).cast()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_uri_equals(first: *mut librdf_uri, second: *mut librdf_uri) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(first) = (unsafe { borrow_handle(first, TAG_URI) }) else {
            return 0;
        };
        let Some(second) = (unsafe { borrow_handle(second, TAG_URI) }) else {
            return 0;
        };
        i32::from(first.inner.node == second.inner.node)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_uri_compare(first: *mut librdf_uri, second: *mut librdf_uri) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(first) = (unsafe { borrow_handle(first, TAG_URI) }) else {
            return 0;
        };
        let Some(second) = (unsafe { borrow_handle(second, TAG_URI) }) else {
            return 0;
        };
        match first.inner.node.as_str().cmp(second.inner.node.as_str()) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_uri_is_file_uri(uri: *mut librdf_uri) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(uri) = (unsafe { borrow_handle(uri, TAG_URI) }) else {
            return 0;
        };
        i32::from(uri.inner.node.as_str().starts_with("file:"))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_uri_to_filename(uri: *mut librdf_uri) -> *mut c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(uri) = (unsafe { borrow_handle(uri, TAG_URI) }) else {
            return ptr::null_mut();
        };
        match file_uri_to_path(uri.inner.node.as_str()) {
            Ok(path) => strdup_c(&path.to_string_lossy()),
            Err(error) => {
                set_last_error(error.to_string());
                ptr::null_mut()
            }
        }
    })
}
