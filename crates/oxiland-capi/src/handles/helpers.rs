//! UTF-8 / path helpers (0.9).

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::cstr_required;
use crate::handles::io::{FILE, writeln_file};
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;

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

#[unsafe(no_mangle)]
pub extern "C" fn librdf_utf8_to_latin1_2(
    input: *const u8,
    length: usize,
    _discard: u8,
    output_length: *mut usize,
) -> *mut u8 {
    librdf_utf8_to_latin1(input, length, output_length)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_latin1_to_utf8_2(
    input: *const u8,
    length: usize,
    output_length: *mut usize,
) -> *mut u8 {
    librdf_latin1_to_utf8(input, length, output_length)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_unicode_char_to_utf8(c: u32, output: *mut u8, length: i32) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(ch) = char::from_u32(c) else {
            set_last_error("invalid unicode scalar");
            return -1;
        };
        let mut buf = [0u8; 4];
        let encoded = ch.encode_utf8(&mut buf).as_bytes();
        if length <= 0 || output.is_null() {
            return encoded.len() as i32;
        }
        if (length as usize) < encoded.len() {
            return 0;
        }
        unsafe { ptr::copy_nonoverlapping(encoded.as_ptr(), output, encoded.len()) };
        encoded.len() as i32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_utf8_to_unicode_char(
    output: *mut u32,
    input: *const u8,
    length: i32,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        if input.is_null() || length <= 0 {
            return -1;
        }
        let bytes = unsafe { std::slice::from_raw_parts(input, length as usize) };
        match std::str::from_utf8(bytes) {
            Ok(s) => {
                let Some(ch) = s.chars().next() else {
                    return -1;
                };
                if !output.is_null() {
                    unsafe { *output = ch as u32 };
                }
                ch.len_utf8() as i32
            }
            Err(_) => -1,
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_utf8_print(input: *const u8, length: i32, stream: *mut FILE) {
    abort_on_panic(|| {
        clear_last_error();
        if input.is_null() || length < 0 {
            return;
        }
        let bytes = unsafe { std::slice::from_raw_parts(input, length as usize) };
        let text = String::from_utf8_lossy(bytes);
        let _ = writeln_file(stream, &text);
    });
}
