//! `librdf_iterator` — opaque void* sequence with optional map callbacks.

use std::os::raw::c_void;
use std::ptr;

use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_ITERATOR, TAG_WORLD, TypedHandle, borrow_handle, box_handle, free_handle,
};

pub type librdf_iterator = TypedHandle<IteratorInner>;

pub type librdf_iterator_map_handler = Option<
    unsafe extern "C" fn(
        iterator: *mut librdf_iterator,
        map_context: *mut c_void,
        item: *mut c_void,
    ) -> *mut c_void,
>;

pub type librdf_iterator_map_free_context_handler =
    Option<unsafe extern "C" fn(map_context: *mut c_void)>;

type IsEndFn = Option<unsafe extern "C" fn(*mut c_void) -> i32>;
type NextFn = Option<unsafe extern "C" fn(*mut c_void) -> i32>;
type GetFn = Option<unsafe extern "C" fn(*mut c_void, i32) -> *mut c_void>;
type FinishedFn = Option<unsafe extern "C" fn(*mut c_void)>;

pub struct IteratorInner {
    /// Materialized items (for list/node iterators).
    pub items: Vec<*mut c_void>,
    pub index: usize,
    /// Optional context/key/value parallel arrays (same length as items, or empty).
    pub contexts: Vec<*mut c_void>,
    pub keys: Vec<*mut c_void>,
    pub values: Vec<*mut c_void>,
    /// Callback-driven iterator state.
    pub context: *mut c_void,
    pub is_end: IsEndFn,
    pub next: NextFn,
    pub get: GetFn,
    pub finished: FinishedFn,
    pub maps: Vec<(
        librdf_iterator_map_handler,
        librdf_iterator_map_free_context_handler,
        *mut c_void,
    )>,
    pub owns_finished: bool,
}

impl Drop for IteratorInner {
    fn drop(&mut self) {
        for (_, free_ctx, ctx) in self.maps.drain(..) {
            if let Some(free_ctx) = free_ctx {
                unsafe { free_ctx(ctx) };
            }
        }
        if self.owns_finished {
            if let Some(finished) = self.finished {
                unsafe { finished(self.context) };
            }
        }
    }
}

impl IteratorInner {
    pub fn from_items(items: Vec<*mut c_void>) -> Self {
        Self {
            items,
            index: 0,
            contexts: Vec::new(),
            keys: Vec::new(),
            values: Vec::new(),
            context: ptr::null_mut(),
            is_end: None,
            next: None,
            get: None,
            finished: None,
            maps: Vec::new(),
            owns_finished: false,
        }
    }

    fn apply_maps(&self, iterator: *mut librdf_iterator, mut item: *mut c_void) -> *mut c_void {
        for (map, _, ctx) in &self.maps {
            if let Some(map) = map {
                item = unsafe { map(iterator, *ctx, item) };
                if item.is_null() {
                    return ptr::null_mut();
                }
            }
        }
        item
    }
}

/// Creates an empty iterator.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_empty_iterator(world: *mut librdf_world) -> *mut librdf_iterator {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        box_handle(TAG_ITERATOR, IteratorInner::from_items(Vec::new()))
    })
}

/// Creates a callback-driven iterator (Redland factory style).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_iterator(
    world: *mut librdf_world,
    context: *mut c_void,
    is_end_method: IsEndFn,
    next_method: NextFn,
    get_method: GetFn,
    finished_method: FinishedFn,
) -> *mut librdf_iterator {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        box_handle(
            TAG_ITERATOR,
            IteratorInner {
                items: Vec::new(),
                index: 0,
                contexts: Vec::new(),
                keys: Vec::new(),
                values: Vec::new(),
                context,
                is_end: is_end_method,
                next: next_method,
                get: get_method,
                finished: finished_method,
                maps: Vec::new(),
                owns_finished: true,
            },
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_iterator(iterator: *mut librdf_iterator) {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { free_handle(iterator, TAG_ITERATOR) };
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_iterator_end(iterator: *mut librdf_iterator) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(it) = (unsafe { borrow_handle(iterator, TAG_ITERATOR) }) else {
            return 1;
        };
        if let Some(is_end) = it.inner.is_end {
            return unsafe { is_end(it.inner.context) };
        }
        i32::from(it.inner.index >= it.inner.items.len())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_iterator_have_elements(iterator: *mut librdf_iterator) -> i32 {
    i32::from(librdf_iterator_end(iterator) == 0)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_iterator_next(iterator: *mut librdf_iterator) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(it) = (unsafe { borrow_handle(iterator, TAG_ITERATOR) }) else {
            return -1;
        };
        if let Some(next) = it.inner.next {
            return unsafe { next(it.inner.context) };
        }
        if it.inner.index >= it.inner.items.len() {
            return 1;
        }
        it.inner.index += 1;
        i32::from(it.inner.index >= it.inner.items.len())
    })
}

fn get_flag(iterator: *mut librdf_iterator, flag: i32) -> *mut c_void {
    abort_on_panic(|| {
        clear_last_error();
        let Some(it) = (unsafe { borrow_handle(iterator, TAG_ITERATOR) }) else {
            return ptr::null_mut();
        };
        if let Some(get) = it.inner.get {
            let item = unsafe { get(it.inner.context, flag) };
            return it.inner.apply_maps(iterator, item);
        }
        if it.inner.index >= it.inner.items.len() {
            return ptr::null_mut();
        }
        let item = match flag {
            0 => it.inner.items[it.inner.index],
            1 => it
                .inner
                .contexts
                .get(it.inner.index)
                .copied()
                .unwrap_or(ptr::null_mut()),
            2 => it
                .inner
                .keys
                .get(it.inner.index)
                .copied()
                .unwrap_or(ptr::null_mut()),
            3 => it
                .inner
                .values
                .get(it.inner.index)
                .copied()
                .unwrap_or(ptr::null_mut()),
            _ => {
                set_last_error("unknown iterator get flag");
                ptr::null_mut()
            }
        };
        it.inner.apply_maps(iterator, item)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_iterator_get_object(iterator: *mut librdf_iterator) -> *mut c_void {
    get_flag(iterator, 0)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_iterator_get_context(iterator: *mut librdf_iterator) -> *mut c_void {
    get_flag(iterator, 1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_iterator_get_key(iterator: *mut librdf_iterator) -> *mut c_void {
    get_flag(iterator, 2)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_iterator_get_value(iterator: *mut librdf_iterator) -> *mut c_void {
    get_flag(iterator, 3)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_iterator_add_map(
    iterator: *mut librdf_iterator,
    map_function: librdf_iterator_map_handler,
    free_context: librdf_iterator_map_free_context_handler,
    map_context: *mut c_void,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(it) = (unsafe { borrow_handle(iterator, TAG_ITERATOR) }) else {
            return -1;
        };
        it.inner
            .maps
            .push((map_function, free_context, map_context));
        0
    })
}

/// Helper used by list/node APIs to box a materialized iterator.
pub fn box_items(items: Vec<*mut c_void>) -> *mut librdf_iterator {
    box_handle(TAG_ITERATOR, IteratorInner::from_items(items))
}
