//! Allocator contract: C strings/buffers freed only via [`librdf_free_memory`].

use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr;

use crate::error::{abort_on_panic, clear_last_error, set_last_error};

/// Allocates a NUL-terminated C string with `libc::malloc` (pair with
/// [`librdf_free_memory`]).
pub fn strdup_c(value: &str) -> *mut c_char {
    let len = value.len().checked_add(1).expect("string too large");
    // SAFETY: malloc returns either null or a block of `len` bytes.
    let ptr = unsafe { libc::malloc(len) }.cast::<c_char>();
    if ptr.is_null() {
        set_last_error("out of memory");
        return ptr::null_mut();
    }
    // SAFETY: `ptr` is a fresh malloc of `len` bytes; copy `value` then NUL.
    unsafe {
        ptr::copy_nonoverlapping(value.as_ptr().cast::<c_char>(), ptr, value.len());
        *ptr.add(value.len()) = 0;
    }
    ptr
}

/// Frees a pointer previously returned by Oxiland C string/buffer APIs.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_memory(ptr: *mut c_void) {
    abort_on_panic(|| {
        clear_last_error();
        if ptr.is_null() {
            return;
        }
        // SAFETY: caller contract — only pointers from Oxiland malloc paths.
        unsafe {
            libc::free(ptr);
        }
    });
}
