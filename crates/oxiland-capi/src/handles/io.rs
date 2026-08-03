//! Shared FILE / iostream helpers for C ABI print/write APIs.
//!
//! Oxiland does not embed Raptor `raptor_iostream`. Instead, iostream
//! arguments that are non-null must be Oxiland tagged handles created via
//! [`oxiland_new_iostream`] / [`oxiland_new_iostream_from_bytes`].

use std::ffi::c_void;
use std::os::raw::c_char;
use std::slice;

use crate::error::set_last_error;
use crate::handles::{TAG_IOSTREAM, TypedHandle, borrow_handle, box_handle, free_handle};

#[allow(
    clippy::upper_case_acronyms,
    reason = "FILE is the spelling of the platform C type"
)]
pub type FILE = libc::FILE;

/// In-memory byte buffer used wherever Redland would pass a `raptor_iostream*`.
pub struct OxilandIostream {
    pub data: Vec<u8>,
}

pub type librdf_iostream = TypedHandle<OxilandIostream>;

/// Creates an empty Oxiland iostream handle (for serialize/write sinks).
pub fn oxiland_new_iostream() -> *mut librdf_iostream {
    box_handle(TAG_IOSTREAM, OxilandIostream { data: Vec::new() })
}

/// Creates an Oxiland iostream handle seeded with `data` (for parse sources).
pub fn oxiland_new_iostream_from_bytes(data: Vec<u8>) -> *mut librdf_iostream {
    box_handle(TAG_IOSTREAM, OxilandIostream { data })
}

/// Frees an Oxiland iostream. Null is a no-op.
pub fn oxiland_free_iostream(iostream: *mut librdf_iostream) {
    // SAFETY: null or a live iostream from oxiland_new_iostream*.
    unsafe { free_handle(iostream, TAG_IOSTREAM) };
}

/// Borrows the accumulated bytes from a live iostream handle.
pub fn oxiland_iostream_data(iostream: *mut librdf_iostream) -> Option<Vec<u8>> {
    // SAFETY: null or a live iostream handle.
    unsafe { borrow_handle(iostream, TAG_IOSTREAM) }.map(|h| h.inner.data.clone())
}

/// Read bytes from an Oxiland iostream handle (`*mut OxilandIostream` tagged).
pub fn read_iostream_bytes(iostream: *mut c_void) -> Result<Vec<u8>, String> {
    if iostream.is_null() {
        return Err("iostream is null".into());
    }
    let ptr = iostream.cast::<librdf_iostream>();
    // SAFETY: pointer is non-null; borrow_handle validates the tag/registry.
    match unsafe { borrow_handle(ptr, TAG_IOSTREAM) } {
        Some(handle) => Ok(handle.inner.data.clone()),
        None => Err("iostream is not an Oxiland iostream handle".into()),
    }
}

/// Write bytes to a C FILE* (null = no-op success).
pub fn write_file(fh: *mut FILE, bytes: &[u8]) -> i32 {
    if fh.is_null() {
        return 0;
    }
    let n = unsafe { libc::fwrite(bytes.as_ptr().cast(), 1, bytes.len(), fh) };
    if n != bytes.len() {
        set_last_error("fwrite failed");
        return -1;
    }
    0
}

/// Write a string plus newline to FILE*.
pub fn writeln_file(fh: *mut FILE, text: &str) -> i32 {
    let mut buf = text.as_bytes().to_vec();
    buf.push(b'\n');
    write_file(fh, &buf)
}

/// Append bytes to an Oxiland iostream handle.
///
/// Null succeeds (same convention as [`write_file`]). Non-null pointers that
/// are not live `TAG_IOSTREAM` handles fail — silent success is not allowed.
pub fn write_iostream(iostr: *mut c_void, bytes: &[u8]) -> i32 {
    if iostr.is_null() {
        return 0;
    }
    let ptr = iostr.cast::<librdf_iostream>();
    // SAFETY: pointer is non-null; borrow_handle validates the tag/registry.
    match unsafe { borrow_handle(ptr, TAG_IOSTREAM) } {
        Some(handle) => {
            handle.inner.data.extend_from_slice(bytes);
            0
        }
        None => {
            set_last_error("iostream is not an Oxiland iostream handle");
            -1
        }
    }
}

#[allow(
    dead_code,
    reason = "shared checked slice helper is retained for C ABI extensions"
)]
/// Returns a borrowed slice over a C buffer.
///
/// # Safety
///
/// For non-null `ptr`, the caller must provide a readable allocation of at
/// least `len` bytes that remains live for the returned lifetime.
pub unsafe fn c_slice<'a>(ptr: *const u8, len: usize) -> Option<&'a [u8]> {
    if ptr.is_null() {
        set_last_error("null buffer");
        None
    } else {
        Some(unsafe { slice::from_raw_parts(ptr, len) })
    }
}

#[allow(
    dead_code,
    reason = "shared checked C-string byte helper is retained for C ABI extensions"
)]
/// Returns the bytes of a borrowed NUL-terminated C string.
///
/// # Safety
///
/// For non-null `ptr`, the caller must provide a valid NUL-terminated string
/// that remains live for the returned lifetime.
pub unsafe fn cstr_bytes<'a>(ptr: *const c_char) -> Option<&'a [u8]> {
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_bytes())
    }
}
