//! UTF-8 / path helpers (0.9).

use std::os::raw::c_char;
use std::path::Path;
use std::ptr;

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::cstr_required;

/// Lossy UTF-8 → Latin-1 (bytes > 255 become `?`).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_utf8_to_latin1(
    input: *const u8,
    length: usize,
    output_length: *mut usize,
) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        if input.is_null() {
            set_last_error("input is null");
            return ptr::null_mut();
        }
        // SAFETY: caller provides `length` readable bytes.
        let slice = unsafe { std::slice::from_raw_parts(input, length) };
        let text = match std::str::from_utf8(slice) {
            Ok(t) => t,
            Err(_) => {
                set_last_error("input is not valid UTF-8");
                return ptr::null_mut();
            }
        };
        let mut out = Vec::with_capacity(text.len());
        for ch in text.chars() {
            if (ch as u32) <= 0xff {
                out.push(ch as u8);
            } else {
                out.push(b'?');
            }
        }
        if !output_length.is_null() {
            unsafe { *output_length = out.len() };
        }
        out.push(0);
        let len = out.len();
        let ptr = unsafe { libc::malloc(len) }.cast::<u8>();
        if ptr.is_null() {
            set_last_error("out of memory");
            return ptr::null_mut();
        }
        unsafe {
            ptr::copy_nonoverlapping(out.as_ptr(), ptr, len);
        }
        ptr
    })
}

/// Latin-1 → UTF-8.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_latin1_to_utf8(
    input: *const u8,
    length: usize,
    output_length: *mut usize,
) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        if input.is_null() {
            set_last_error("input is null");
            return ptr::null_mut();
        }
        let slice = unsafe { std::slice::from_raw_parts(input, length) };
        let text: String = slice.iter().map(|&b| b as char).collect();
        let bytes = text.into_bytes();
        if !output_length.is_null() {
            unsafe { *output_length = bytes.len() };
        }
        let mut out = bytes;
        out.push(0);
        let len = out.len();
        let ptr = unsafe { libc::malloc(len) }.cast::<u8>();
        if ptr.is_null() {
            set_last_error("out of memory");
            return ptr::null_mut();
        }
        unsafe {
            ptr::copy_nonoverlapping(out.as_ptr(), ptr, len);
        }
        ptr
    })
}

/// Returns malloc'd basename of `name`.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_basename(name: *const c_char) -> *mut c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(name) = (unsafe { cstr_required(name, "name") }) else {
            return ptr::null_mut();
        };
        let base = Path::new(name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(name);
        strdup_c(base)
    })
}
