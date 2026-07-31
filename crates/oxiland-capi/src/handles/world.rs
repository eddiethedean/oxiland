//! `librdf_world` handle.

use std::os::raw::c_char;
use std::sync::Mutex;

use oxiland::{LogFacility, LogLevel, World};

use crate::error::{abort_on_panic, clear_last_error};
use crate::handles::{
    TAG_WORLD, TypedHandle, borrow_handle, box_handle, cstr_optional, free_handle,
};

pub type librdf_world = TypedHandle<WorldInner>;

pub type librdf_log_func = Option<
    unsafe extern "C" fn(
        user_data: *mut std::ffi::c_void,
        code: i32,
        level: i32,
        facility: i32,
        message: *const c_char,
        locator: *const c_char,
    ) -> i32,
>;

pub struct WorldInner {
    pub world: World,
    pub opened: bool,
    pub logger: Mutex<Option<(librdf_log_func, *mut std::ffi::c_void)>>,
}

/// Creates a new world handle.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_world() -> *mut librdf_world {
    abort_on_panic(|| {
        clear_last_error();
        box_handle(
            TAG_WORLD,
            WorldInner {
                world: World::new(),
                opened: false,
                logger: Mutex::new(None),
            },
        )
    })
}

/// Frees a world. Null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_world(world: *mut librdf_world) {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { free_handle(world, TAG_WORLD) };
    });
}

/// Opens the world (marks opened; construction already initializes).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_open(world: *mut librdf_world) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(handle) = (unsafe { borrow_handle(world, TAG_WORLD) }) else {
            return;
        };
        handle.inner.opened = true;
    });
}

/// Registers a C log callback. Pass a null `logger` to clear.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_set_logger(
    world: *mut librdf_world,
    user_data: *mut std::ffi::c_void,
    logger: librdf_log_func,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(handle) = (unsafe { borrow_handle(world, TAG_WORLD) }) else {
            return -1;
        };
        *handle
            .inner
            .logger
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = if logger.is_some() {
            Some((logger, user_data))
        } else {
            None
        };
        0
    })
}

/// Emits a simple log message through the world logger / Oxiland World.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_log_simple(
    world: *mut librdf_world,
    code: i32,
    level: i32,
    facility: i32,
    message: *const c_char,
) {
    abort_on_panic(|| {
        clear_last_error();
        let logger = {
            let Some(handle) = (unsafe { borrow_handle(world, TAG_WORLD) }) else {
                return;
            };
            let msg = match unsafe { cstr_optional(message, "message") } {
                Ok(Some(m)) => m.to_owned(),
                Ok(None) => String::new(),
                Err(()) => return,
            };
            let ox_level = match level {
                0 => LogLevel::Debug,
                1 => LogLevel::Info,
                2 => LogLevel::Warn,
                _ => LogLevel::Error,
            };
            handle
                .inner
                .world
                .log(ox_level, LogFacility::General, msg.clone());
            let logger = *handle
                .inner
                .logger
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            (logger, msg)
        };
        if let (Some((Some(cb), user_data)), msg) = logger {
            let cmsg = std::ffi::CString::new(msg).unwrap_or_default();
            unsafe {
                cb(
                    user_data,
                    code,
                    level,
                    facility,
                    cmsg.as_ptr(),
                    std::ptr::null(),
                );
            }
        }
    });
}
