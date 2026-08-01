//! `librdf_list` — Vec-backed opaque pointer list.

use std::os::raw::c_void;
use std::ptr;

use crate::error::{abort_on_panic, clear_last_error};
use crate::handles::iterator::{box_items, librdf_iterator};
use crate::handles::world::librdf_world;
use crate::handles::{TAG_LIST, TAG_WORLD, TypedHandle, borrow_handle, box_handle, free_handle};

pub type librdf_list = TypedHandle<ListInner>;

pub type ListEqualsFn = Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> i32>;

pub struct ListInner {
    pub items: Vec<*mut c_void>,
    pub equals: ListEqualsFn,
}

fn ptr_eq(a: *mut c_void, b: *mut c_void, equals: ListEqualsFn) -> bool {
    if let Some(equals) = equals {
        unsafe { equals(a, b) != 0 }
    } else {
        a == b
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_list(world: *mut librdf_world) -> *mut librdf_list {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        box_handle(
            TAG_LIST,
            ListInner {
                items: Vec::new(),
                equals: None,
            },
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_list(list: *mut librdf_list) {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { free_handle(list, TAG_LIST) };
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_list_clear(list: *mut librdf_list) {
    abort_on_panic(|| {
        clear_last_error();
        if let Some(list) = unsafe { borrow_handle(list, TAG_LIST) } {
            list.inner.items.clear();
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_list_add(list: *mut librdf_list, data: *mut c_void) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(list) = (unsafe { borrow_handle(list, TAG_LIST) }) else {
            return -1;
        };
        list.inner.items.push(data);
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_list_unshift(list: *mut librdf_list, data: *mut c_void) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(list) = (unsafe { borrow_handle(list, TAG_LIST) }) else {
            return -1;
        };
        list.inner.items.insert(0, data);
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_list_shift(list: *mut librdf_list) -> *mut c_void {
    abort_on_panic(|| {
        clear_last_error();
        let Some(list) = (unsafe { borrow_handle(list, TAG_LIST) }) else {
            return ptr::null_mut();
        };
        if list.inner.items.is_empty() {
            ptr::null_mut()
        } else {
            list.inner.items.remove(0)
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_list_pop(list: *mut librdf_list) -> *mut c_void {
    abort_on_panic(|| {
        clear_last_error();
        let Some(list) = (unsafe { borrow_handle(list, TAG_LIST) }) else {
            return ptr::null_mut();
        };
        list.inner.items.pop().unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_list_remove(list: *mut librdf_list, data: *mut c_void) -> *mut c_void {
    abort_on_panic(|| {
        clear_last_error();
        let Some(list) = (unsafe { borrow_handle(list, TAG_LIST) }) else {
            return ptr::null_mut();
        };
        let equals = list.inner.equals;
        if let Some(pos) = list
            .inner
            .items
            .iter()
            .position(|&item| ptr_eq(item, data, equals))
        {
            list.inner.items.remove(pos)
        } else {
            ptr::null_mut()
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_list_contains(list: *mut librdf_list, data: *mut c_void) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(list) = (unsafe { borrow_handle(list, TAG_LIST) }) else {
            return 0;
        };
        let equals = list.inner.equals;
        i32::from(
            list.inner
                .items
                .iter()
                .any(|&item| ptr_eq(item, data, equals)),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_list_size(list: *mut librdf_list) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(list) = (unsafe { borrow_handle(list, TAG_LIST) }) else {
            return -1;
        };
        i32::try_from(list.inner.items.len()).unwrap_or(i32::MAX)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_list_set_equals(list: *mut librdf_list, equals: ListEqualsFn) {
    abort_on_panic(|| {
        clear_last_error();
        if let Some(list) = unsafe { borrow_handle(list, TAG_LIST) } {
            list.inner.equals = equals;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_list_get_iterator(list: *mut librdf_list) -> *mut librdf_iterator {
    abort_on_panic(|| {
        clear_last_error();
        let Some(list) = (unsafe { borrow_handle(list, TAG_LIST) }) else {
            return ptr::null_mut();
        };
        box_items(list.inner.items.clone())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_list_foreach(
    list: *mut librdf_list,
    fn_: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
    user_data: *mut c_void,
) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(list) = (unsafe { borrow_handle(list, TAG_LIST) }) else {
            return;
        };
        let Some(fn_) = fn_ else {
            return;
        };
        for &item in &list.inner.items {
            unsafe { fn_(item, user_data) };
        }
    });
}
