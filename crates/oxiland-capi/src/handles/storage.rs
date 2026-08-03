//! `librdf_storage` handle.

use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::hash::librdf_hash;
use crate::handles::iterator::librdf_iterator;
use crate::handles::model::librdf_model;
use crate::handles::model::{
    librdf_model_add_statement, librdf_model_add_statements, librdf_model_as_stream,
    librdf_model_contains_statement, librdf_model_context_add_statement,
    librdf_model_context_add_statements, librdf_model_context_as_stream,
    librdf_model_context_remove_statement, librdf_model_context_remove_statements,
    librdf_model_find_statements, librdf_model_find_statements_in_context,
    librdf_model_find_statements_with_options, librdf_model_get_arcs, librdf_model_get_arcs_in,
    librdf_model_get_arcs_out, librdf_model_get_contexts, librdf_model_get_feature,
    librdf_model_get_sources, librdf_model_get_targets, librdf_model_has_arc_in,
    librdf_model_has_arc_out, librdf_model_remove_statement, librdf_model_set_feature,
    librdf_model_size, librdf_model_transaction_commit, librdf_model_transaction_get_handle,
    librdf_model_transaction_rollback, librdf_model_transaction_start,
    librdf_model_transaction_start_with_handle,
};
use crate::handles::node::librdf_node;
use crate::handles::query::{librdf_model_query_execute, librdf_query, librdf_query_results};
use crate::handles::statement::librdf_statement;
use crate::handles::stream::librdf_stream;
use crate::handles::uri::librdf_uri;
use crate::handles::world::{librdf_world, register_baseline_storage, reject_factory_callback};
use crate::handles::{
    TAG_STORAGE, TAG_WORLD, TypedHandle, borrow_handle, box_handle, cstr_optional, cstr_required,
    free_handle,
};
use oxiland::StorageBackend;
use std::ffi::c_void;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr;

pub type librdf_storage = TypedHandle<StorageInner>;

pub struct StorageInner {
    pub backend: StorageBackend,
    pub path: Option<PathBuf>,
    pub opened: bool,
    pub world: *mut librdf_world,
    pub model: Option<*mut librdf_model>,
    pub refs: i32,
    pub instance: *mut std::ffi::c_void,
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
        let Some(_world_handle) = (unsafe { borrow_handle(world, TAG_WORLD) }) else {
            return ptr::null_mut();
        };
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
                world,
                model: None,
                refs: 1,
                instance: ptr::null_mut(),
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

fn associated_model(storage: *mut librdf_storage) -> Option<*mut librdf_model> {
    let handle = unsafe { borrow_handle(storage, TAG_STORAGE) }?;
    handle.inner.model
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_storage_with_options(
    world: *mut librdf_world,
    storage_name: *const c_char,
    name: *const c_char,
    _options: *mut librdf_hash,
) -> *mut librdf_storage {
    librdf_new_storage(world, storage_name, name, ptr::null())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_storage_from_storage(
    old_storage: *mut librdf_storage,
) -> *mut librdf_storage {
    abort_on_panic(|| {
        clear_last_error();
        let Some(old) = (unsafe { borrow_handle(old_storage, TAG_STORAGE) }) else {
            return ptr::null_mut();
        };
        box_handle(
            TAG_STORAGE,
            StorageInner {
                backend: old.inner.backend,
                path: old.inner.path.clone(),
                opened: false,
                world: old.inner.world,
                model: None,
                refs: 1,
                instance: ptr::null_mut(),
            },
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_storage_from_factory(
    world: *mut librdf_world,
    factory: *mut c_void,
    name: *const c_char,
    options_string: *const c_char,
) -> *mut librdf_storage {
    let _ = factory;
    librdf_new_storage(world, c"memory".as_ptr(), name, options_string)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_add_reference(storage: *mut librdf_storage) {
    abort_on_panic(|| {
        clear_last_error();
        if let Some(s) = unsafe { borrow_handle(storage, TAG_STORAGE) } {
            s.inner.refs = s.inner.refs.saturating_add(1);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_remove_reference(storage: *mut librdf_storage) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(s) = (unsafe { borrow_handle(storage, TAG_STORAGE) }) else {
            return;
        };
        s.inner.refs = s.inner.refs.saturating_sub(1);
        if s.inner.refs <= 0 {
            unsafe { free_handle(storage, TAG_STORAGE) };
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_close(storage: *mut librdf_storage) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(s) = (unsafe { borrow_handle(storage, TAG_STORAGE) }) else {
            return -1;
        };
        s.inner.opened = false;
        s.inner.model = None;
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_size(storage: *mut librdf_storage) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_size(m))
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_add_statement(
    storage: *mut librdf_storage,
    statement: *mut librdf_statement,
) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_add_statement(m, statement))
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_add_statements(
    storage: *mut librdf_storage,
    statement_stream: *mut librdf_stream,
) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_add_statements(m, statement_stream))
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_remove_statement(
    storage: *mut librdf_storage,
    statement: *mut librdf_statement,
) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_remove_statement(m, statement))
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_contains_statement(
    storage: *mut librdf_storage,
    statement: *mut librdf_statement,
) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_contains_statement(m, statement))
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_serialise(storage: *mut librdf_storage) -> *mut librdf_stream {
    associated_model(storage)
        .map(|m| librdf_model_as_stream(m))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_find_statements(
    storage: *mut librdf_storage,
    statement: *mut librdf_statement,
) -> *mut librdf_stream {
    associated_model(storage)
        .map(|m| librdf_model_find_statements(m, statement))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_find_statements_in_context(
    storage: *mut librdf_storage,
    statement: *mut librdf_statement,
    context_node: *mut librdf_node,
) -> *mut librdf_stream {
    associated_model(storage)
        .map(|m| librdf_model_find_statements_in_context(m, statement, context_node))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_find_statements_with_options(
    storage: *mut librdf_storage,
    statement: *mut librdf_statement,
    context_node: *mut librdf_node,
    options: *mut librdf_hash,
) -> *mut librdf_stream {
    associated_model(storage)
        .map(|m| librdf_model_find_statements_with_options(m, statement, context_node, options))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_get_sources(
    storage: *mut librdf_storage,
    arc: *mut librdf_node,
    target: *mut librdf_node,
) -> *mut librdf_iterator {
    associated_model(storage)
        .map(|m| librdf_model_get_sources(m, arc, target))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_get_arcs(
    storage: *mut librdf_storage,
    source: *mut librdf_node,
    target: *mut librdf_node,
) -> *mut librdf_iterator {
    associated_model(storage)
        .map(|m| librdf_model_get_arcs(m, source, target))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_get_targets(
    storage: *mut librdf_storage,
    source: *mut librdf_node,
    arc: *mut librdf_node,
) -> *mut librdf_iterator {
    associated_model(storage)
        .map(|m| librdf_model_get_targets(m, source, arc))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_get_arcs_in(
    storage: *mut librdf_storage,
    node: *mut librdf_node,
) -> *mut librdf_iterator {
    associated_model(storage)
        .map(|m| librdf_model_get_arcs_in(m, node))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_get_arcs_out(
    storage: *mut librdf_storage,
    node: *mut librdf_node,
) -> *mut librdf_iterator {
    associated_model(storage)
        .map(|m| librdf_model_get_arcs_out(m, node))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_has_arc_in(
    storage: *mut librdf_storage,
    node: *mut librdf_node,
    property: *mut librdf_node,
) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_has_arc_in(m, node, property))
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_has_arc_out(
    storage: *mut librdf_storage,
    node: *mut librdf_node,
    property: *mut librdf_node,
) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_has_arc_out(m, node, property))
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_context_add_statement(
    storage: *mut librdf_storage,
    context: *mut librdf_node,
    statement: *mut librdf_statement,
) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_context_add_statement(m, context, statement))
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_context_add_statements(
    storage: *mut librdf_storage,
    context: *mut librdf_node,
    stream: *mut librdf_stream,
) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_context_add_statements(m, context, stream))
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_context_remove_statement(
    storage: *mut librdf_storage,
    context: *mut librdf_node,
    statement: *mut librdf_statement,
) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_context_remove_statement(m, context, statement))
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_context_remove_statements(
    storage: *mut librdf_storage,
    context: *mut librdf_node,
) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_context_remove_statements(m, context))
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_context_as_stream(
    storage: *mut librdf_storage,
    context: *mut librdf_node,
) -> *mut librdf_stream {
    associated_model(storage)
        .map(|m| librdf_model_context_as_stream(m, context))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_context_serialise(
    storage: *mut librdf_storage,
    context: *mut librdf_node,
) -> *mut librdf_stream {
    librdf_storage_context_as_stream(storage, context)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_get_contexts(
    storage: *mut librdf_storage,
) -> *mut librdf_iterator {
    associated_model(storage)
        .map(|m| librdf_model_get_contexts(m))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_get_feature(
    storage: *mut librdf_storage,
    feature: *mut librdf_uri,
) -> *mut librdf_node {
    associated_model(storage)
        .map(|m| librdf_model_get_feature(m, feature))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_set_feature(
    storage: *mut librdf_storage,
    feature: *mut librdf_uri,
    value: *mut librdf_node,
) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_set_feature(m, feature, value))
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_get_world(storage: *mut librdf_storage) -> *mut librdf_world {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { borrow_handle(storage, TAG_STORAGE) }
            .map(|s| s.inner.world)
            .unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_get_instance(storage: *mut librdf_storage) -> *mut c_void {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { borrow_handle(storage, TAG_STORAGE) }
            .map(|s| s.inner.instance)
            .unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_set_instance(storage: *mut librdf_storage, instance: *mut c_void) {
    abort_on_panic(|| {
        clear_last_error();
        if let Some(s) = unsafe { borrow_handle(storage, TAG_STORAGE) } {
            s.inner.instance = instance;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_supports_query(
    storage: *mut librdf_storage,
    _query: *mut librdf_query,
) -> i32 {
    i32::from(associated_model(storage).is_some())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_query_execute(
    storage: *mut librdf_storage,
    query: *mut librdf_query,
) -> *mut librdf_query_results {
    associated_model(storage)
        .map(|m| librdf_model_query_execute(m, query))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_transaction_start(storage: *mut librdf_storage) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_transaction_start(m))
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_transaction_start_with_handle(
    storage: *mut librdf_storage,
    handle: *mut c_void,
) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_transaction_start_with_handle(m, handle))
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_transaction_commit(storage: *mut librdf_storage) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_transaction_commit(m))
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_transaction_rollback(storage: *mut librdf_storage) -> i32 {
    associated_model(storage)
        .map(|m| librdf_model_transaction_rollback(m))
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_transaction_get_handle(
    storage: *mut librdf_storage,
) -> *mut c_void {
    associated_model(storage)
        .map(|m| librdf_model_transaction_get_handle(m))
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_storage_register_factory(
    world: *mut librdf_world,
    name: *const c_char,
    _label: *const c_char,
    factory: Option<unsafe extern "C" fn(*mut c_void)>,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(handle) = (unsafe { borrow_handle(world, TAG_WORLD) }) else {
            return -1;
        };
        let Some(name) = (unsafe { cstr_required(name, "name") }) else {
            return -1;
        };
        if reject_factory_callback(factory) {
            set_last_error(
                "storage factory callbacks are unsupported; register baseline names only",
            );
            return -1;
        }
        match register_baseline_storage(&mut handle.inner, name) {
            Ok(()) => 0,
            Err(error) => {
                set_last_error(error);
                -1
            }
        }
    })
}
