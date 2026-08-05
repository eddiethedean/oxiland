//! `librdf_stream` handle.

use crate::error::{abort_on_panic, clear_last_error, clear_last_error_if_set, set_last_error};
use crate::handles::io::FILE;
use crate::handles::iterator::librdf_iterator;
use crate::handles::node::librdf_node;
use crate::handles::statement::{StatementInner, librdf_statement};
use crate::handles::world::librdf_world;
use crate::handles::{TAG_ITERATOR, TAG_NODE, TAG_WORLD};
use crate::handles::{
    TAG_STATEMENT, TAG_STREAM, TypedHandle, borrow_handle, borrow_handle_hot, box_handle,
    free_handle,
};
use oxigraph::model::Triple;
use oxiland::StatementMatches;
use std::ffi::c_void;
use std::ptr;

pub type librdf_stream = TypedHandle<StreamInner>;

pub struct StreamInner {
    pub statements: Vec<StatementInner>,
    /// Raw triples for model-backed streams. `StatementInner` conversion is
    /// deferred until `librdf_stream_get_object` actually observes an item.
    pub triples: Vec<Triple>,
    /// Lazy store cursor. When set, `lookahead` holds the current triple.
    pub matches: Option<StatementMatches>,
    pub lookahead: Option<Triple>,
    /// Exact length for full-model snapshots. When present, end/next can use
    /// index arithmetic and defer decoding store rows until get_object.
    pub known_len: Option<usize>,
    /// Number of rows already pulled from `matches`.
    pub matches_consumed: usize,
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

impl StreamInner {
    pub fn from_matches(mut matches: StatementMatches) -> Result<Self, String> {
        let lookahead = match matches.next() {
            None => None,
            Some(Ok(quad)) => Some(Triple::new(quad.subject, quad.predicate, quad.object)),
            Some(Err(error)) => return Err(error.to_string()),
        };
        let matches_consumed = usize::from(lookahead.is_some());
        Ok(Self {
            statements: Vec::new(),
            triples: Vec::new(),
            matches: Some(matches),
            lookahead,
            known_len: None,
            matches_consumed,
            index: 0,
            current: None,
        })
    }

    pub fn from_matches_with_len(matches: StatementMatches, known_len: usize) -> Self {
        Self {
            statements: Vec::new(),
            triples: Vec::new(),
            matches: Some(matches),
            lookahead: None,
            known_len: Some(known_len),
            matches_consumed: 0,
            index: 0,
            current: None,
        }
    }

    pub fn from_triples(triples: Vec<Triple>) -> Self {
        let known_len = triples.len();
        Self {
            statements: Vec::new(),
            triples,
            matches: None,
            lookahead: None,
            known_len: Some(known_len),
            matches_consumed: 0,
            index: 0,
            current: None,
        }
    }

    pub fn from_statements(statements: Vec<StatementInner>) -> Self {
        Self {
            statements,
            triples: Vec::new(),
            matches: None,
            lookahead: None,
            known_len: None,
            matches_consumed: 0,
            index: 0,
            current: None,
        }
    }

    fn is_lazy(&self) -> bool {
        self.matches.is_some() || self.lookahead.is_some()
    }

    fn materialized_len(&self) -> usize {
        debug_assert!(self.statements.is_empty() || self.triples.is_empty());
        self.statements.len() + self.triples.len()
    }

    fn at_end(&self) -> bool {
        if let Some(known_len) = self.known_len {
            return self.index >= known_len;
        }
        if self.is_lazy() {
            self.lookahead.is_none() && self.matches.is_none()
        } else {
            self.index >= self.materialized_len()
        }
    }

    fn advance_lazy(&mut self) -> Result<bool, String> {
        self.drop_current();
        self.index = self.index.saturating_add(1);
        self.lookahead = None;
        if self.known_len.is_some() {
            return Ok(self.at_end());
        }
        let next = match self.matches.as_mut() {
            Some(matches) => matches.next(),
            None => None,
        };
        match next {
            None => {
                self.matches = None;
                self.lookahead = None;
                Ok(true)
            }
            Some(Ok(quad)) => {
                self.matches_consumed += 1;
                self.lookahead = Some(Triple::new(quad.subject, quad.predicate, quad.object));
                Ok(false)
            }
            Some(Err(error)) => Err(error.to_string()),
        }
    }

    fn drop_current(&mut self) {
        if let Some(ptr) = self.current.take() {
            if !ptr.is_null() {
                // SAFETY: previous current owned exclusively by this stream.
                unsafe { free_handle(ptr, TAG_STATEMENT) };
            }
        }
    }

    fn ensure_current(&mut self) -> Result<(), String> {
        if self.current.is_some() || self.at_end() {
            return Ok(());
        }
        if self.lookahead.is_none() && self.matches.is_some() {
            while self.matches_consumed <= self.index {
                let next = self.matches.as_mut().and_then(Iterator::next);
                match next {
                    Some(Ok(quad)) => {
                        self.matches_consumed += 1;
                        self.lookahead =
                            Some(Triple::new(quad.subject, quad.predicate, quad.object));
                    }
                    Some(Err(error)) => return Err(error.to_string()),
                    None => {
                        self.matches = None;
                        self.lookahead = None;
                        return Ok(());
                    }
                }
            }
        }
        let statement = if let Some(triple) = self.lookahead.as_ref() {
            Some(StatementInner::from_triple(triple.clone()))
        } else {
            self.statements.get(self.index).cloned().or_else(|| {
                self.triples
                    .get(self.index)
                    .cloned()
                    .map(StatementInner::from_triple)
            })
        };
        if let Some(stmt) = statement {
            self.current = Some(box_handle(TAG_STATEMENT, stmt));
        }
        Ok(())
    }
}

/// Returns nonzero if the stream is finished.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_stream_end(stream: *mut librdf_stream) -> i32 {
    // SAFETY: null or live stream handle from this crate.
    if let Some(stream) = unsafe { borrow_handle_hot(stream, TAG_STREAM) } {
        return i32::from(stream.inner.at_end());
    }
    abort_on_panic(|| {
        clear_last_error_if_set();
        // SAFETY: stream is null or a live stream handle.
        let Some(stream) = (unsafe { borrow_handle(stream, TAG_STREAM) }) else {
            return 1;
        };
        i32::from(stream.inner.at_end())
    })
}

/// Advances the stream. Returns nonzero on error / past end.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_stream_next(stream: *mut librdf_stream) -> i32 {
    // SAFETY: null or live stream handle from this crate.
    if let Some(stream) = unsafe { borrow_handle_hot(stream, TAG_STREAM) } {
        if stream.inner.is_lazy() {
            return match stream.inner.advance_lazy() {
                Ok(true) => 1,
                Ok(false) => 0,
                Err(_) => -1,
            };
        }
        if stream.inner.at_end() {
            return 1;
        }
        stream.inner.drop_current();
        stream.inner.index += 1;
        return i32::from(stream.inner.at_end());
    }
    abort_on_panic(|| {
        clear_last_error_if_set();
        // SAFETY: stream is null or a live stream handle.
        let Some(stream) = (unsafe { borrow_handle(stream, TAG_STREAM) }) else {
            return -1;
        };
        if stream.inner.is_lazy() {
            return match stream.inner.advance_lazy() {
                Ok(true) => 1,
                Ok(false) => 0,
                Err(error) => {
                    set_last_error(error);
                    -1
                }
            };
        }
        if stream.inner.at_end() {
            return 1;
        }
        stream.inner.drop_current();
        stream.inner.index += 1;
        i32::from(stream.inner.at_end())
    })
}

/// Returns the current statement (owned by the stream; do not free).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_stream_get_object(stream: *mut librdf_stream) -> *mut librdf_statement {
    abort_on_panic(|| {
        clear_last_error_if_set();
        // SAFETY: stream is null or a live stream handle.
        let Some(stream) = (unsafe { borrow_handle(stream, TAG_STREAM) }) else {
            return ptr::null_mut();
        };
        if stream.inner.at_end() {
            return ptr::null_mut();
        }
        if let Err(error) = stream.inner.ensure_current() {
            set_last_error(error);
            return ptr::null_mut();
        }
        stream.inner.current.unwrap_or(ptr::null_mut())
    })
}

/// Frees a stream. Null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_stream(stream: *mut librdf_stream) {
    abort_on_panic(|| {
        clear_last_error_if_set();
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
        box_handle(TAG_STREAM, StreamInner::from_statements(Vec::new()))
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
        box_handle(TAG_STREAM, StreamInner::from_statements(statements))
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
