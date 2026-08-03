//! Allocator contract: C strings/buffers freed only via [`librdf_free_memory`].

use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr;

use crate::error::{abort_on_panic, clear_last_error, set_last_error};

/// Allocates a NUL-terminated C string with `libc::malloc` (pair with
/// [`librdf_free_memory`]).
pub fn strdup_c(value: &str) -> *mut c_char {
    let Some(len) = value.len().checked_add(1) else {
        set_last_error("string allocation size overflow");
        return ptr::null_mut();
    };
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

/// Allocates `size` bytes (pair with [`librdf_free_memory`]).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_alloc_memory(size: usize) -> *mut c_void {
    abort_on_panic(|| {
        clear_last_error();
        if size == 0 {
            return ptr::null_mut();
        }
        // SAFETY: `malloc` accepts any nonzero size and returns either null or
        // an allocation owned by the caller.
        let p = unsafe { libc::malloc(size) };
        if p.is_null() {
            set_last_error("out of memory");
        }
        p
    })
}

/// Allocates and zeroes `nmemb * size` bytes.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_calloc_memory(nmemb: usize, size: usize) -> *mut c_void {
    abort_on_panic(|| {
        clear_last_error();
        let Some(bytes) = nmemb.checked_mul(size) else {
            set_last_error("allocation size overflow");
            return ptr::null_mut();
        };
        if bytes == 0 {
            return ptr::null_mut();
        }
        // SAFETY: multiplication was checked above; `calloc` returns either
        // null or a zeroed allocation owned by the caller.
        let p = unsafe { libc::calloc(nmemb, size) };
        if p.is_null() {
            set_last_error("out of memory");
        }
        p
    })
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
