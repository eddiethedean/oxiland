//! `librdf_world` handle.

use crate::error::{abort_on_panic, clear_last_error};
use crate::handles::node::{NodeInner, librdf_node};
use crate::handles::uri::librdf_uri;
use crate::handles::{TAG_NODE, TAG_URI};
use crate::handles::{
    TAG_WORLD, TypedHandle, borrow_handle, box_handle, cstr_optional, free_handle,
};
use oxigraph::model::Term;
use oxiland::{LogFacility, LogLevel, World};
use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Mutex;

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

pub type librdf_log_level_func =
    Option<unsafe extern "C" fn(user_data: *mut std::ffi::c_void, message: *const c_char)>;

pub type librdf_raptor_init_handler = Option<
    unsafe extern "C" fn(user_data: *mut std::ffi::c_void, raptor_world: *mut std::ffi::c_void),
>;
pub type librdf_rasqal_init_handler = Option<
    unsafe extern "C" fn(user_data: *mut std::ffi::c_void, rasqal_world: *mut std::ffi::c_void),
>;

/// Redland-shaped factory init callback (`void (*)(librdf_*_factory*)`).
///
/// ADR-025: Oxiland does **not** execute caller-supplied factory callbacks. The
/// type is retained for signature compatibility with Redland registration APIs.
pub type FactoryInitFn = Option<unsafe extern "C" fn(*mut c_void)>;

/// Stored `librdf_*_register_factory` entry (name only; callbacks are never run).
pub struct RegisteredFactory {
    pub name: String,
}

pub struct WorldInner {
    pub world: World,
    pub opened: bool,
    pub logger: Mutex<Option<(librdf_log_func, *mut std::ffi::c_void)>>,
    pub raptor: *mut std::ffi::c_void,
    pub rasqal: *mut std::ffi::c_void,
    pub digest_name: Option<String>,
    pub features: std::collections::HashMap<String, String>,
    pub error_handler: Option<(librdf_log_level_func, *mut std::ffi::c_void)>,
    pub warning_handler: Option<(librdf_log_level_func, *mut std::ffi::c_void)>,
    pub raptor_init: Option<(librdf_raptor_init_handler, *mut std::ffi::c_void)>,
    pub rasqal_init: Option<(librdf_rasqal_init_handler, *mut std::ffi::c_void)>,
    pub registered_parsers: Vec<String>,
    pub registered_serializers: Vec<String>,
    pub registered_storages: Vec<String>,
    pub registered_queries: Vec<String>,
    pub parser_factories: std::collections::HashMap<String, RegisteredFactory>,
    pub serializer_factories: std::collections::HashMap<String, RegisteredFactory>,
    pub storage_factories: std::collections::HashMap<String, RegisteredFactory>,
    pub query_factories: std::collections::HashMap<String, RegisteredFactory>,
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
                raptor: std::ptr::null_mut(),
                rasqal: std::ptr::null_mut(),
                digest_name: None,
                features: std::collections::HashMap::new(),
                error_handler: None,
                warning_handler: None,
                raptor_init: None,
                rasqal_init: None,
                registered_parsers: Vec::new(),
                registered_serializers: Vec::new(),
                registered_storages: Vec::new(),
                registered_queries: Vec::new(),
                parser_factories: std::collections::HashMap::new(),
                serializer_factories: std::collections::HashMap::new(),
                storage_factories: std::collections::HashMap::new(),
                query_factories: std::collections::HashMap::new(),
            },
        )
    })
}

/// ADR-025: do not execute caller-supplied factory callbacks (fake cookies are UB).
pub(crate) fn reject_factory_callback(factory: FactoryInitFn) -> bool {
    factory.is_some()
}

/// Records a baseline factory name on the world after validating via oxiland::factory.
pub(crate) fn register_baseline_parser(
    handle: &mut WorldInner,
    name: &str,
) -> Result<(), String> {
    oxiland::factory::register_parser_factory(name).map_err(|e| e.to_string())?;
    let key = name.to_ascii_lowercase();
    handle.registered_parsers.push(name.to_owned());
    handle
        .parser_factories
        .insert(key, RegisteredFactory { name: name.to_owned() });
    Ok(())
}

pub(crate) fn register_baseline_serializer(
    handle: &mut WorldInner,
    name: &str,
) -> Result<(), String> {
    oxiland::factory::register_serializer_factory(name).map_err(|e| e.to_string())?;
    let key = name.to_ascii_lowercase();
    handle.registered_serializers.push(name.to_owned());
    handle
        .serializer_factories
        .insert(key, RegisteredFactory { name: name.to_owned() });
    Ok(())
}

pub(crate) fn register_baseline_storage(
    handle: &mut WorldInner,
    name: &str,
) -> Result<(), String> {
    oxiland::factory::register_storage_factory(name).map_err(|e| e.to_string())?;
    let key = name.to_ascii_lowercase();
    handle.registered_storages.push(name.to_owned());
    handle
        .storage_factories
        .insert(key, RegisteredFactory { name: name.to_owned() });
    Ok(())
}

pub(crate) fn register_baseline_query(
    handle: &mut WorldInner,
    name: &str,
) -> Result<(), String> {
    oxiland::factory::register_query_factory(name).map_err(|e| e.to_string())?;
    let key = name.to_ascii_lowercase();
    handle.registered_queries.push(name.to_owned());
    handle
        .query_factories
        .insert(key, RegisteredFactory { name: name.to_owned() });
    Ok(())
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
    locator: *mut c_void,
    message: *const c_char,
) {
    abort_on_panic(|| {
        clear_last_error();
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
        let error_handler = handle.inner.error_handler;
        let warning_handler = handle.inner.warning_handler;
        let cmsg = std::ffi::CString::new(msg).unwrap_or_default();
        if let Some((Some(cb), user_data)) = logger {
            // SAFETY: callback registered by librdf_world_set_logger.
            unsafe {
                cb(
                    user_data,
                    code,
                    level,
                    facility,
                    cmsg.as_ptr(),
                    locator.cast(),
                );
            }
        }
        if level >= 3 {
            if let Some((Some(cb), user_data)) = error_handler {
                unsafe { cb(user_data, cmsg.as_ptr()) };
            }
        } else if level == 2 {
            if let Some((Some(cb), user_data)) = warning_handler {
                unsafe { cb(user_data, cmsg.as_ptr()) };
            }
        }
    });
}

static GLOBAL_WORLD: AtomicPtr<librdf_world> = AtomicPtr::new(std::ptr::null_mut());

#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_init_mutex(world: *mut librdf_world) {
    abort_on_panic(|| {
        clear_last_error();
        let _ = unsafe { borrow_handle(world, TAG_WORLD) };
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_set_raptor(world: *mut librdf_world, raptor_world_ptr: *mut c_void) {
    abort_on_panic(|| {
        clear_last_error();
        if let Some(handle) = unsafe { borrow_handle(world, TAG_WORLD) } {
            handle.inner.raptor = raptor_world_ptr;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_get_raptor(world: *mut librdf_world) -> *mut c_void {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { borrow_handle(world, TAG_WORLD) }
            .map(|h| h.inner.raptor)
            .unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_set_rasqal(world: *mut librdf_world, rasqal_world_ptr: *mut c_void) {
    abort_on_panic(|| {
        clear_last_error();
        if let Some(handle) = unsafe { borrow_handle(world, TAG_WORLD) } {
            handle.inner.rasqal = rasqal_world_ptr;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_get_rasqal(world: *mut librdf_world) -> *mut c_void {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { borrow_handle(world, TAG_WORLD) }
            .map(|h| h.inner.rasqal)
            .unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_set_raptor_init_handler(
    world: *mut librdf_world,
    user_data: *mut c_void,
    handler: librdf_raptor_init_handler,
) {
    abort_on_panic(|| {
        clear_last_error();
        if let Some(handle) = unsafe { borrow_handle(world, TAG_WORLD) } {
            handle.inner.raptor_init = Some((handler, user_data));
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_set_rasqal_init_handler(
    world: *mut librdf_world,
    user_data: *mut c_void,
    handler: librdf_rasqal_init_handler,
) {
    abort_on_panic(|| {
        clear_last_error();
        if let Some(handle) = unsafe { borrow_handle(world, TAG_WORLD) } {
            handle.inner.rasqal_init = Some((handler, user_data));
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_set_error(
    world: *mut librdf_world,
    user_data: *mut c_void,
    error_handler: librdf_log_level_func,
) {
    abort_on_panic(|| {
        clear_last_error();
        if let Some(handle) = unsafe { borrow_handle(world, TAG_WORLD) } {
            handle.inner.error_handler = Some((error_handler, user_data));
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_set_warning(
    world: *mut librdf_world,
    user_data: *mut c_void,
    warning_handler: librdf_log_level_func,
) {
    abort_on_panic(|| {
        clear_last_error();
        if let Some(handle) = unsafe { borrow_handle(world, TAG_WORLD) } {
            handle.inner.warning_handler = Some((warning_handler, user_data));
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_set_digest(world: *mut librdf_world, name: *const c_char) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(handle) = (unsafe { borrow_handle(world, TAG_WORLD) }) else {
            return;
        };
        handle.inner.digest_name = unsafe { cstr_optional(name, "name") }
            .ok()
            .flatten()
            .map(str::to_owned);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_get_feature(
    world: *mut librdf_world,
    feature: *mut librdf_uri,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        let Some(handle) = (unsafe { borrow_handle(world, TAG_WORLD) }) else {
            return ptr::null_mut();
        };
        let Some(feature) = (unsafe { borrow_handle(feature, TAG_URI) }) else {
            return ptr::null_mut();
        };
        let key = feature.inner.node.as_str();
        match handle.inner.features.get(key) {
            Some(v) => box_handle(
                TAG_NODE,
                NodeInner::from_term(Term::Literal(oxigraph::model::Literal::new_simple_literal(
                    v,
                ))),
            ),
            None => ptr::null_mut(),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_set_feature(
    world: *mut librdf_world,
    feature: *mut librdf_uri,
    value: *mut librdf_node,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(handle) = (unsafe { borrow_handle(world, TAG_WORLD) }) else {
            return -1;
        };
        let Some(feature) = (unsafe { borrow_handle(feature, TAG_URI) }) else {
            return -1;
        };
        let Some(value) = (unsafe { borrow_handle(value, TAG_NODE) }) else {
            return -1;
        };
        let text = match &value.inner.term {
            Term::Literal(lit) => lit.value().to_owned(),
            Term::NamedNode(n) => n.as_str().to_owned(),
            Term::BlankNode(b) => b.as_str().to_owned(),
            #[allow(unreachable_patterns)]
            _ => value.inner.term.to_string(),
        };
        handle
            .inner
            .features
            .insert(feature.inner.node.as_str().to_owned(), text.clone());
        handle.inner.world.set_feature(
            feature.inner.node.as_str(),
            oxiland::FeatureValue::String(text),
        );
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_init_world(digest_factory_name: *mut c_char, _not_used2: *mut c_void) {
    abort_on_panic(|| {
        clear_last_error();
        if !GLOBAL_WORLD.load(Ordering::SeqCst).is_null() {
            return;
        }
        let world = librdf_new_world();
        if !digest_factory_name.is_null() {
            librdf_world_set_digest(world, digest_factory_name);
        }
        librdf_world_open(world);
        GLOBAL_WORLD.store(world, Ordering::SeqCst);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_destroy_world() {
    abort_on_panic(|| {
        clear_last_error();
        let world = GLOBAL_WORLD.swap(ptr::null_mut(), Ordering::SeqCst);
        librdf_free_world(world);
    });
}
