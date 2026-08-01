//! Shared FILE / iostream helpers for C ABI print/write APIs.

use std::ffi::c_void;
use std::os::raw::c_char;
use std::slice;

use crate::error::set_last_error;

#[allow(clippy::upper_case_acronyms)]
pub type FILE = libc::FILE;

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

/// Best-effort write to opaque raptor_iostream* stored as void*.
/// When null, succeeds. When non-null, treats as unused and returns 0
/// (Oxiland does not embed Raptor iostreams).
pub fn write_iostream(_iostr: *mut c_void, _bytes: &[u8]) -> i32 {
    0
}

#[allow(dead_code)]
pub unsafe fn c_slice<'a>(ptr: *const u8, len: usize) -> Option<&'a [u8]> {
    if ptr.is_null() {
        set_last_error("null buffer");
        None
    } else {
        Some(unsafe { slice::from_raw_parts(ptr, len) })
    }
}

#[allow(dead_code)]
pub unsafe fn cstr_bytes<'a>(ptr: *const c_char) -> Option<&'a [u8]> {
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_bytes())
    }
}
