//! `librdf_stream` handle.

use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::io::FILE;
use crate::handles::iterator::librdf_iterator;
use crate::handles::node::librdf_node;
use crate::handles::statement::{StatementInner, librdf_statement};
use crate::handles::world::librdf_world;
use crate::handles::{TAG_ITERATOR, TAG_NODE, TAG_WORLD};
use crate::handles::{
    TAG_STATEMENT, TAG_STREAM, TypedHandle, borrow_handle, box_handle, free_handle,
};
use oxigraph::model::Triple;
use std::ffi::c_void;
use std::ptr;

pub type librdf_stream = TypedHandle<StreamInner>;

pub struct StreamInner {
    pub statements: Vec<StatementInner>,
    /// Raw triples for model-backed streams. `StatementInner` conversion is
    /// deferred until `librdf_stream_get_object` actually observes an item.
    pub triples: Vec<Triple>,
    pub index: usize,
    /// Borrowed-by-C current object; owned by the stream.
    pub current: Option<*mut librdf_statement>,
}

impl Drop for StreamInner {
    fn drop(&mut self) {
        if let Some(ptr) = self.current.take() {
            if !ptr.is_null() {
                // SAFETY: current was boxed by this stream and remains owned here.
                unsafe { free_handle(ptr, TAG_STATEMENT) };
            }
        }
    }
}

fn refresh_current(stream: &mut StreamInner) {
    if let Some(ptr) = stream.current.take() {
        if !ptr.is_null() {
            // SAFETY: previous current owned exclusively by this stream.
            unsafe { free_handle(ptr, TAG_STATEMENT) };
        }
    }
    let statement = stream.statements.get(stream.index).cloned().or_else(|| {
        stream
            .triples
            .get(stream.index)
            .cloned()
            .map(StatementInner::from_triple)
    });
    if let Some(stmt) = statement {
        stream.current = Some(box_handle(TAG_STATEMENT, stmt));
    }
}

fn stream_len(stream: &StreamInner) -> usize {
    debug_assert!(stream.statements.is_empty() || stream.triples.is_empty());
    stream.statements.len() + stream.triples.len()
}

/// Returns nonzero if the stream is finished.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_stream_end(stream: *mut librdf_stream) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: stream is null or a live stream handle.
        let Some(stream) = (unsafe { borrow_handle(stream, TAG_STREAM) }) else {
            return 1;
        };
        i32::from(stream.inner.index >= stream_len(&stream.inner))
    })
}

/// Advances the stream. Returns nonzero on error / past end.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_stream_next(stream: *mut librdf_stream) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: stream is null or a live stream handle.
        let Some(stream) = (unsafe { borrow_handle(stream, TAG_STREAM) }) else {
            return -1;
        };
        if stream.inner.index >= stream_len(&stream.inner) {
            return 1;
        }
        if let Some(ptr) = stream.inner.current.take() {
            if !ptr.is_null() {
                // SAFETY: the current statement is owned exclusively by this stream.
                unsafe { free_handle(ptr, TAG_STATEMENT) };
            }
        }
        stream.inner.index += 1;
        if stream.inner.index >= stream_len(&stream.inner) {
            1
        } else {
            0
        }
    })
}

/// Returns the current statement (owned by the stream; do not free).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_stream_get_object(stream: *mut librdf_stream) -> *mut librdf_statement {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: stream is null or a live stream handle.
        let Some(stream) = (unsafe { borrow_handle(stream, TAG_STREAM) }) else {
            return ptr::null_mut();
        };
        if stream.inner.index >= stream_len(&stream.inner) {
            return ptr::null_mut();
        }
        if stream.inner.current.is_none() {
            refresh_current(&mut stream.inner);
        }
        stream.inner.current.unwrap_or(ptr::null_mut())
    })
}

/// Frees a stream. Null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_stream(stream: *mut librdf_stream) {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: stream is null or a live stream handle.
        unsafe { free_handle(stream, TAG_STREAM) };
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_empty_stream(world: *mut librdf_world) -> *mut librdf_stream {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        box_handle(
            TAG_STREAM,
            StreamInner {
                statements: Vec::new(),
                triples: Vec::new(),
                index: 0,
                current: None,
            },
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_stream(
    world: *mut librdf_world,
    _context: *mut c_void,
    _is_end_method: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    _next_method: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    _get_method: Option<unsafe extern "C" fn(*mut c_void, i32) -> *mut c_void>,
    _finished_method: Option<unsafe extern "C" fn(*mut c_void)>,
) -> *mut librdf_stream {
    // Callback-driven streams accepted; materialize as empty until mapped.
    librdf_new_empty_stream(world)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_stream_from_node_iterator(
    iterator: *mut librdf_iterator,
    statement: *mut librdf_statement,
    field: u32,
) -> *mut librdf_stream {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return ptr::null_mut();
        };
        if unsafe { borrow_handle(iterator, TAG_ITERATOR) }.is_none() {
            return ptr::null_mut();
        }
        let mut statements = Vec::new();
        while crate::handles::iterator::librdf_iterator_end(iterator) == 0 {
            let obj = crate::handles::iterator::librdf_iterator_get_object(iterator);
            let mut stmt = statement.inner.clone();
            if let Some(node) = unsafe { borrow_handle(obj.cast::<librdf_node>(), TAG_NODE) } {
                let n = Some(node.inner.clone());
                match field {
                    1 => stmt.subject = n,
                    2 => stmt.predicate = n,
                    _ => stmt.object = n,
                }
            }
            statements.push(stmt);
            if crate::handles::iterator::librdf_iterator_next(iterator) != 0 {
                break;
            }
        }
        box_handle(
            TAG_STREAM,
            StreamInner {
                statements,
                triples: Vec::new(),
                index: 0,
                current: None,
            },
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_stream_get_context(stream: *mut librdf_stream) -> *mut c_void {
    let _ = stream;
    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_stream_get_context2(stream: *mut librdf_stream) -> *mut librdf_node {
    let _ = stream;
    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_stream_add_map(
    stream: *mut librdf_stream,
    _map_function: *mut c_void,
    _free_context: *mut c_void,
    _map_context: *mut c_void,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(stream, TAG_STREAM) }.is_none() {
            return -1;
        }
        set_last_error("librdf_stream_add_map is unsupported");
        -1
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_stream_print(stream: *mut librdf_stream, fh: *mut FILE) {
    abort_on_panic(|| {
        clear_last_error();
        while librdf_stream_end(stream) == 0 {
            let stmt = librdf_stream_get_object(stream);
            crate::handles::statement::librdf_statement_print(stmt, fh);
            if librdf_stream_next(stream) != 0 {
                break;
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_stream_write(stream: *mut librdf_stream, iostr: *mut c_void) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        while librdf_stream_end(stream) == 0 {
            let stmt = librdf_stream_get_object(stream);
            if crate::handles::statement::librdf_statement_write(stmt, iostr) != 0 {
                return -1;
            }
            if librdf_stream_next(stream) != 0 {
                break;
            }
        }
        0
    })
}
