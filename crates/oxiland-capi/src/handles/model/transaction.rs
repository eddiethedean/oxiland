//! Transaction-state entry points for `librdf_model`.

use super::librdf_model;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::{TAG_MODEL, borrow_handle};
use std::ffi::c_void;
use std::ptr;

fn begin(model: *mut librdf_model) -> i32 {
    let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
        return -1;
    };
    if !model.inner.transaction.begin() {
        set_last_error("transaction already active");
        return -1;
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_transaction_start(model: *mut librdf_model) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        begin(model)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_transaction_start_with_handle(
    model: *mut librdf_model,
    handle: *mut c_void,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        model.inner.transaction.begin_with_handle(handle);
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_transaction_commit(model: *mut librdf_model) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        model.inner.transaction.finish();
        match model.inner.model.sync() {
            Ok(()) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_transaction_rollback(model: *mut librdf_model) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        model.inner.transaction.finish();
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_transaction_get_handle(model: *mut librdf_model) -> *mut c_void {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { borrow_handle(model, TAG_MODEL) }
            .map(|model| model.inner.transaction.handle())
            .unwrap_or(ptr::null_mut())
    })
}
