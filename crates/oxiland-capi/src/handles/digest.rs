//! Digest handles (0.9).

use std::os::raw::c_char;
use std::ptr;

use oxiland::utility::{DigestAlgorithm, digest_bytes};

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_DIGEST, TAG_WORLD, TypedHandle, borrow_handle, box_handle, cstr_required, free_handle,
};

pub type librdf_digest = TypedHandle<DigestInner>;

pub struct DigestInner {
    pub algorithm: DigestAlgorithm,
    pub buffer: Vec<u8>,
    pub finalized: Option<Vec<u8>>,
    pub hex: Option<*mut c_char>,
}

impl Drop for DigestInner {
    fn drop(&mut self) {
        if let Some(ptr) = self.hex.take() {
            if !ptr.is_null() {
                // SAFETY: allocated via strdup_c.
                unsafe { libc::free(ptr.cast()) };
            }
        }
    }
}

/// Creates a digest (`md5`, `sha1`, or `sha256`).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_digest(
    world: *mut librdf_world,
    name: *const c_char,
) -> *mut librdf_digest {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        let Some(name) = (unsafe { cstr_required(name, "name") }) else {
            return ptr::null_mut();
        };
        match DigestAlgorithm::from_name(name) {
            Ok(algorithm) => box_handle(
                TAG_DIGEST,
                DigestInner {
                    algorithm,
                    buffer: Vec::new(),
                    finalized: None,
                    hex: None,
                },
            ),
            Err(error) => {
                set_last_error(error.to_string());
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_digest(digest: *mut librdf_digest) {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { free_handle(digest, TAG_DIGEST) };
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_digest_init(digest: *mut librdf_digest) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(digest) = (unsafe { borrow_handle(digest, TAG_DIGEST) }) else {
            return;
        };
        digest.inner.buffer.clear();
        digest.inner.finalized = None;
        if let Some(ptr) = digest.inner.hex.take() {
            if !ptr.is_null() {
                unsafe { libc::free(ptr.cast()) };
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_digest_update(
    digest: *mut librdf_digest,
    buffer: *const u8,
    length: usize,
) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(digest) = (unsafe { borrow_handle(digest, TAG_DIGEST) }) else {
            return;
        };
        if length == 0 {
            return;
        }
        if buffer.is_null() {
            set_last_error("buffer is null with nonzero length");
            return;
        }
        // SAFETY: caller provides `length` readable bytes.
        let slice = unsafe { std::slice::from_raw_parts(buffer, length) };
        digest.inner.buffer.extend_from_slice(slice);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_digest_update_string(digest: *mut librdf_digest, string: *const u8) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(digest) = (unsafe { borrow_handle(digest, TAG_DIGEST) }) else {
            return;
        };
        let Some(string) = (unsafe { cstr_required(string.cast(), "string") }) else {
            return;
        };
        digest.inner.buffer.extend_from_slice(string.as_bytes());
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_digest_final(digest: *mut librdf_digest) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(digest) = (unsafe { borrow_handle(digest, TAG_DIGEST) }) else {
            return;
        };
        let bytes = digest_bytes(digest.inner.algorithm, &digest.inner.buffer);
        digest.inner.finalized = Some(bytes);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_digest_to_string(digest: *mut librdf_digest) -> *mut c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(digest) = (unsafe { borrow_handle(digest, TAG_DIGEST) }) else {
            return ptr::null_mut();
        };
        let Some(finalized) = digest.inner.finalized.as_ref() else {
            set_last_error("digest not finalized");
            return ptr::null_mut();
        };
        let hex = finalized
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        if let Some(ptr) = digest.inner.hex.take() {
            if !ptr.is_null() {
                unsafe { libc::free(ptr.cast()) };
            }
        }
        let ptr = strdup_c(&hex);
        digest.inner.hex = Some(ptr);
        ptr
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_digest_get_digest(digest: *mut librdf_digest) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(digest) = (unsafe { borrow_handle(digest, TAG_DIGEST) }) else {
            return ptr::null_mut();
        };
        match digest.inner.finalized.as_mut() {
            Some(bytes) => bytes.as_mut_ptr(),
            None => {
                set_last_error("digest not finalized");
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_digest_get_digest_length(digest: *mut librdf_digest) -> usize {
    abort_on_panic(|| -> usize {
        clear_last_error();
        let Some(digest) = (unsafe { borrow_handle(digest, TAG_DIGEST) }) else {
            return 0usize;
        };
        digest
            .inner
            .finalized
            .as_ref()
            .map(|b| b.len())
            .unwrap_or(0usize)
    })
}
