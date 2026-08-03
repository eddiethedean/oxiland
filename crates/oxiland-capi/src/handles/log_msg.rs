//! `librdf_log` and log-message field accessors.

use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr;

use crate::error::{abort_on_panic, clear_last_error};
use crate::handles::world::{librdf_log_simple, librdf_world};
use crate::handles::{TAG_WORLD, borrow_handle};

/// Redland-shaped log message blob (stable layout for accessors).
#[repr(C)]
pub struct librdf_log_message {
    pub code: i32,
    pub level: i32,
    pub facility: i32,
    pub message: *const c_char,
    pub locator: *mut c_void,
}

#[unsafe(no_mangle)]
pub extern "C" fn oxiland_librdf_log_fixed(
    world: *mut librdf_world,
    code: i32,
    level: i32,
    facility: i32,
    locator: *mut c_void,
    message: *const c_char,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return -1;
        }
        librdf_log_simple(world, code, level, facility, locator, message);
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_log_message_code(message: *mut librdf_log_message) -> i32 {
    if message.is_null() {
        return 0;
    }
    unsafe { (*message).code }
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_log_message_level(message: *mut librdf_log_message) -> i32 {
    if message.is_null() {
        return 0;
    }
    unsafe { (*message).level }
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_log_message_facility(message: *mut librdf_log_message) -> i32 {
    if message.is_null() {
        return 0;
    }
    unsafe { (*message).facility }
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_log_message_message(message: *mut librdf_log_message) -> *const c_char {
    if message.is_null() {
        return ptr::null();
    }
    unsafe { (*message).message }
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_log_message_locator(message: *mut librdf_log_message) -> *mut c_void {
    if message.is_null() {
        return ptr::null_mut();
    }
    unsafe { (*message).locator }
}
