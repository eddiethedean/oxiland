//! `librdf_stream` handle.

use std::ptr;

use crate::error::{abort_on_panic, clear_last_error};
use crate::handles::statement::{StatementInner, librdf_statement};
use crate::handles::{
    TAG_STATEMENT, TAG_STREAM, TypedHandle, borrow_handle, box_handle, free_handle,
};

pub type librdf_stream = TypedHandle<StreamInner>;

pub struct StreamInner {
    pub statements: Vec<StatementInner>,
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
    if stream.index < stream.statements.len() {
        let stmt = stream.statements[stream.index].clone();
        stream.current = Some(box_handle(TAG_STATEMENT, stmt));
    }
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
        if stream.inner.index >= stream.inner.statements.len() {
            1
        } else {
            if stream.inner.current.is_none() {
                refresh_current(&mut stream.inner);
            }
            0
        }
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
        if stream.inner.index >= stream.inner.statements.len() {
            return 1;
        }
        stream.inner.index += 1;
        refresh_current(&mut stream.inner);
        if stream.inner.index >= stream.inner.statements.len() {
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
        if stream.inner.index >= stream.inner.statements.len() {
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
