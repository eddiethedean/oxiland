//! Heuristic helpers from Redland.

use std::os::raw::c_char;
use std::ptr;

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error};
use crate::handles::cstr_required;

#[unsafe(no_mangle)]
pub extern "C" fn librdf_heuristic_gen_name(name: *const c_char) -> *mut c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(name) = (unsafe { cstr_required(name, "name") }) else {
            return ptr::null_mut();
        };
        // Redland appends a generated suffix; Oxiland uses a simple counter-free unique suffix.
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        strdup_c(&format!("{name}{n}"))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_heuristic_is_blank_node(node: *const c_char) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { cstr_required(node, "node") }) else {
            return 0;
        };
        i32::from(node.starts_with("_:"))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_heuristic_get_blank_node(node: *const c_char) -> *const c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { cstr_required(node, "node") }) else {
            return ptr::null();
        };
        if node.starts_with("_:") {
            // Return pointer into the caller string past "_:"
            unsafe { node.as_ptr().add(2).cast() }
        } else {
            ptr::null()
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_heuristic_object_is_literal(object: *const c_char) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(object) = (unsafe { cstr_required(object, "object") }) else {
            return 0;
        };
        // Redland heuristic: not a URI-looking token and not a blank.
        i32::from(
            !(object.starts_with("http:")
                || object.starts_with("https:")
                || object.starts_with("_:")),
        )
    })
}
