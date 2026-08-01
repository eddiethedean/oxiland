//! `librdf_uri` handle.

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::io::{FILE, writeln_file};
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_URI, TAG_WORLD, TypedHandle, borrow_handle, box_handle, cstr_required, free_handle,
};
use oxigraph::model::NamedNode;
use oxiland::utility::file_uri_to_path;
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;

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

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_uri2(
    world: *mut librdf_world,
    uri_string: *const c_char,
    length: usize,
) -> *mut librdf_uri {
    abort_on_panic(|| {
        clear_last_error();
        if uri_string.is_null() {
            set_last_error("uri_string is null");
            return ptr::null_mut();
        }
        let bytes = unsafe { std::slice::from_raw_parts(uri_string.cast::<u8>(), length) };
        let s = match std::str::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => {
                set_last_error("uri_string not UTF-8");
                return ptr::null_mut();
            }
        };
        let c = match std::ffi::CString::new(s) {
            Ok(c) => c,
            Err(_) => {
                set_last_error("uri_string contains NUL");
                return ptr::null_mut();
            }
        };
        librdf_new_uri(world, c.as_ptr())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_uri_from_uri(old_uri: *mut librdf_uri) -> *mut librdf_uri {
    abort_on_panic(|| {
        clear_last_error();
        let Some(old) = (unsafe { borrow_handle(old_uri, TAG_URI) }) else {
            return ptr::null_mut();
        };
        box_handle(TAG_URI, UriInner::new(old.inner.node.clone()))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_uri_from_uri_local_name(
    old_uri: *mut librdf_uri,
    local_name: *const c_char,
) -> *mut librdf_uri {
    abort_on_panic(|| {
        clear_last_error();
        let Some(old) = (unsafe { borrow_handle(old_uri, TAG_URI) }) else {
            return ptr::null_mut();
        };
        let Some(local) = (unsafe { cstr_required(local_name, "local_name") }) else {
            return ptr::null_mut();
        };
        match NamedNode::new(format!("{}{local}", old.inner.node.as_str())) {
            Ok(n) => box_handle(TAG_URI, UriInner::new(n)),
            Err(e) => {
                set_last_error(e.to_string());
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_uri_from_filename(
    world: *mut librdf_world,
    filename: *const c_char,
) -> *mut librdf_uri {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        let Some(filename) = (unsafe { cstr_required(filename, "filename") }) else {
            return ptr::null_mut();
        };
        let path = Path::new(filename);
        let iri = if let Ok(canon) = std::fs::canonicalize(path) {
            format!("file://{}", canon.display())
        } else if filename.starts_with('/') {
            format!("file://{filename}")
        } else {
            format!("file:{filename}")
        };
        match NamedNode::new(iri.replace('\\', "/")) {
            Ok(n) => box_handle(TAG_URI, UriInner::new(n)),
            Err(e) => {
                set_last_error(e.to_string());
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_uri_normalised_to_base(
    uri_string: *const c_char,
    source_uri: *mut librdf_uri,
    base_uri: *mut librdf_uri,
) -> *mut librdf_uri {
    let _ = source_uri;
    abort_on_panic(|| {
        clear_last_error();
        let Some(base) = (unsafe { borrow_handle(base_uri, TAG_URI) }) else {
            return ptr::null_mut();
        };
        let Some(uri_string) = (unsafe { cstr_required(uri_string, "uri_string") }) else {
            return ptr::null_mut();
        };
        // Absolute stays; relative appends to base.
        let iri = if uri_string.contains("://") {
            uri_string.to_owned()
        } else {
            format!("{}{uri_string}", base.inner.node.as_str())
        };
        match NamedNode::new(iri) {
            Ok(n) => box_handle(TAG_URI, UriInner::new(n)),
            Err(e) => {
                set_last_error(e.to_string());
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_uri_relative_to_base(
    base_uri: *mut librdf_uri,
    uri_string: *const c_char,
) -> *mut librdf_uri {
    librdf_new_uri_normalised_to_base(uri_string, ptr::null_mut(), base_uri)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_uri_as_counted_string(
    uri: *mut librdf_uri,
    len_p: *mut usize,
) -> *const c_char {
    let p = librdf_uri_as_string(uri);
    if !p.is_null() && !len_p.is_null() {
        let s = unsafe { std::ffi::CStr::from_ptr(p) };
        unsafe { *len_p = s.to_bytes().len() };
    }
    p
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_uri_to_counted_string(uri: *mut librdf_uri, len_p: *mut usize) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(uri) = (unsafe { borrow_handle(uri, TAG_URI) }) else {
            return ptr::null_mut();
        };
        let text = uri.inner.node.as_str();
        if !len_p.is_null() {
            unsafe { *len_p = text.len() };
        }
        strdup_c(text).cast()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_uri_print(uri: *mut librdf_uri, fh: *mut FILE) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(uri) = (unsafe { borrow_handle(uri, TAG_URI) }) else {
            return;
        };
        let _ = writeln_file(fh, uri.inner.node.as_str());
    });
}
