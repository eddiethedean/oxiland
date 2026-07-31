//! `librdf_storage` handle.

use std::path::PathBuf;
use std::ptr;

use oxiland::StorageBackend;

use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::model::librdf_model;
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_STORAGE, TAG_WORLD, TypedHandle, borrow_handle, box_handle, cstr_optional, cstr_required,
    free_handle,
};

pub type librdf_storage = TypedHandle<StorageInner>;

pub struct StorageInner {
    pub backend: StorageBackend,
    pub path: Option<PathBuf>,
    pub opened: bool,
}

/// Creates storage (`"memory"` or `"fjall"`; fjall `name` is the path).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_storage(
    world: *mut librdf_world,
    storage_name: *const std::os::raw::c_char,
    name: *const std::os::raw::c_char,
    _options: *const std::os::raw::c_char,
) -> *mut librdf_storage {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: world is null or a live world handle.
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        // SAFETY: storage_name is a C string when non-null.
        let Some(storage_name) = (unsafe { cstr_required(storage_name, "storage_name") }) else {
            return ptr::null_mut();
        };
        let backend = match StorageBackend::from_name(storage_name) {
            Ok(b) => b,
            Err(error) => {
                set_last_error(error.to_string());
                return ptr::null_mut();
            }
        };
        // SAFETY: name is optional C string.
        let path = match unsafe { cstr_optional(name, "name") } {
            Ok(Some(p)) => Some(PathBuf::from(p)),
            Ok(None) => None,
            Err(()) => return ptr::null_mut(),
        };
        if backend != StorageBackend::Memory && path.is_none() {
            set_last_error(format!("{} storage requires a path name", backend.name()));
            return ptr::null_mut();
        }
        box_handle(
            TAG_STORAGE,
            StorageInner {
                backend,
                path,
                opened: false,
            },
        )
    })
}

/// Frees storage. Null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_storage(storage: *mut librdf_storage) {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: storage is null or a live storage handle.
        unsafe { free_handle(storage, TAG_STORAGE) };
    });
}

/// Opens storage (preview: marks opened; model creation opens the engine).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_open(
    storage: *mut librdf_storage,
    _model: *mut librdf_model,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: storage is null or a live storage handle.
        let Some(handle) = (unsafe { borrow_handle(storage, TAG_STORAGE) }) else {
            return -1;
        };
        handle.inner.opened = true;
        0
    })
}

/// Enumerates compiled storage backends. Returns nonzero when `counter` is valid.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_enumerate(
    _world: *mut librdf_world,
    counter: u32,
    name: *mut *const std::os::raw::c_char,
    label: *mut *const std::os::raw::c_char,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let backends = oxiland::compiled_backends();
        let Ok(index) = usize::try_from(counter) else {
            return 0;
        };
        let Some(backend) = backends.get(index) else {
            return 0;
        };
        let cname: &'static std::ffi::CStr = match backend.name() {
            "memory" => c"memory",
            "fjall" => c"fjall",
            "redb" => c"redb",
            "rocksdb" => c"rocksdb",
            "sqlite" => c"sqlite",
            "lmdb" => c"lmdb",
            _ => c"unknown",
        };
        if !name.is_null() {
            unsafe { *name = cname.as_ptr() };
        }
        if !label.is_null() {
            unsafe { *label = cname.as_ptr() };
        }
        1
    })
}

/// Storage sync is a no-op at the storage handle layer (use model sync).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_sync(storage: *mut librdf_storage) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(storage, TAG_STORAGE) }.is_none() {
            return -1;
        }
        0
    })
}
