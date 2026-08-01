//! Thread-local last-error and panic containment for the C ABI.

use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Records a diagnostic message for the current thread.
pub fn set_last_error(message: impl Into<String>) {
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = Some(message.into());
    });
}

/// Clears the thread-local last-error slot.
pub fn clear_last_error() {
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = None;
    });
}

/// Returns the last error message, if any.
#[allow(dead_code)]
#[must_use]
pub fn last_error() -> Option<String> {
    LAST_ERROR.with(|slot| slot.borrow().clone())
}

/// Default return values when an `extern "C"` body panics.
pub trait FfiDefault {
    fn ffi_default() -> Self;
}

impl FfiDefault for i32 {
    fn ffi_default() -> Self {
        -1
    }
}

impl FfiDefault for usize {
    fn ffi_default() -> Self {
        0
    }
}

impl FfiDefault for i64 {
    fn ffi_default() -> Self {
        -1
    }
}

impl FfiDefault for u32 {
    fn ffi_default() -> Self {
        0
    }
}

impl FfiDefault for () {
    fn ffi_default() -> Self {}
}

impl<T> FfiDefault for *mut T {
    fn ffi_default() -> Self {
        ptr::null_mut()
    }
}

impl<T> FfiDefault for *const T {
    fn ffi_default() -> Self {
        ptr::null()
    }
}

/// Runs `f` inside `catch_unwind`. Panics become last-error + [`FfiDefault`].
pub fn abort_on_panic<T: FfiDefault>(f: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => value,
        Err(_) => {
            set_last_error("internal panic");
            T::ffi_default()
        }
    }
}
