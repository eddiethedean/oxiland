//! `librdf_statement` handle.

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error};
use crate::handles::io::{FILE, write_iostream, writeln_file};
use crate::handles::node::librdf_node_encode;
use crate::handles::node::{NodeInner, librdf_node, take_node};
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_NODE, TAG_STATEMENT, TAG_WORLD, TypedHandle, borrow_handle, box_handle, free_handle,
};
use oxigraph::model::{NamedOrBlankNode, Term, Triple};
use std::ffi::c_void;
use std::ptr;

pub type librdf_statement = TypedHandle<StatementInner>;

#[derive(Clone, Debug, Default)]
pub struct StatementInner {
    pub subject: Option<NodeInner>,
    pub predicate: Option<NodeInner>,
    pub object: Option<NodeInner>,
    subject_ptr: Option<*mut librdf_node>,
    predicate_ptr: Option<*mut librdf_node>,
    object_ptr: Option<*mut librdf_node>,
}

impl Drop for StatementInner {
    fn drop(&mut self) {
        for ptr in [
            self.subject_ptr.take(),
            self.predicate_ptr.take(),
            self.object_ptr.take(),
        ]
        .into_iter()
        .flatten()
        {
            if !ptr.is_null() {
                unsafe { free_handle(ptr, TAG_NODE) };
            }
        }
    }
}

impl StatementInner {
    pub fn from_triple(triple: Triple) -> Self {
        Self {
            subject: Some(NodeInner::from_term(match triple.subject {
                NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n),
                NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b),
            })),
            predicate: Some(NodeInner::from_term(Term::NamedNode(triple.predicate))),
            object: Some(NodeInner::from_term(triple.object)),
            subject_ptr: None,
            predicate_ptr: None,
            object_ptr: None,
        }
    }

    pub fn clear_cached(&mut self) {
        for ptr in [
            self.subject_ptr.take(),
            self.predicate_ptr.take(),
            self.object_ptr.take(),
        ]
        .into_iter()
        .flatten()
        {
            if !ptr.is_null() {
                unsafe { free_handle(ptr, TAG_NODE) };
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_statement(world: *mut librdf_world) -> *mut librdf_statement {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        box_handle(TAG_STATEMENT, StatementInner::default())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_statement_from_nodes(
    world: *mut librdf_world,
    subject: *mut librdf_node,
    predicate: *mut librdf_node,
    object: *mut librdf_node,
) -> *mut librdf_statement {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        let subject = unsafe { take_node(subject) };
        let predicate = unsafe { take_node(predicate) };
        let object = unsafe { take_node(object) };
        box_handle(
            TAG_STATEMENT,
            StatementInner {
                subject,
                predicate,
                object,
                subject_ptr: None,
                predicate_ptr: None,
                object_ptr: None,
            },
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_statement(statement: *mut librdf_statement) {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { free_handle(statement, TAG_STATEMENT) };
    });
}

/// Clears all nodes from a statement, releasing any statement-owned cached nodes.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_clear(statement: *mut librdf_statement) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return;
        };
        statement.inner.clear_cached();
        statement.inner.subject = None;
        statement.inner.predicate = None;
        statement.inner.object = None;
    });
}

fn cached_node(
    cache: &mut Option<*mut librdf_node>,
    value: &Option<NodeInner>,
) -> *mut librdf_node {
    if let Some(ptr) = *cache {
        return ptr;
    }
    let Some(inner) = value else {
        return ptr::null_mut();
    };
    let ptr = box_handle(TAG_NODE, inner.clone());
    *cache = Some(ptr);
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_get_subject(
    statement: *mut librdf_statement,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return ptr::null_mut();
        };
        cached_node(&mut statement.inner.subject_ptr, &statement.inner.subject)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_get_predicate(
    statement: *mut librdf_statement,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return ptr::null_mut();
        };
        cached_node(
            &mut statement.inner.predicate_ptr,
            &statement.inner.predicate,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_get_object(
    statement: *mut librdf_statement,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return ptr::null_mut();
        };
        cached_node(&mut statement.inner.object_ptr, &statement.inner.object)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_set_subject(
    statement: *mut librdf_statement,
    node: *mut librdf_node,
) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return;
        };
        statement.inner.clear_cached();
        statement.inner.subject = unsafe { take_node(node) };
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_set_predicate(
    statement: *mut librdf_statement,
    node: *mut librdf_node,
) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return;
        };
        statement.inner.clear_cached();
        statement.inner.predicate = unsafe { take_node(node) };
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_set_object(
    statement: *mut librdf_statement,
    node: *mut librdf_node,
) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return;
        };
        statement.inner.clear_cached();
        statement.inner.object = unsafe { take_node(node) };
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_equals(
    first: *mut librdf_statement,
    second: *mut librdf_statement,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(first) = (unsafe { borrow_handle(first, TAG_STATEMENT) }) else {
            return 0;
        };
        let Some(second) = (unsafe { borrow_handle(second, TAG_STATEMENT) }) else {
            return 0;
        };
        i32::from(
            first.inner.subject == second.inner.subject
                && first.inner.predicate == second.inner.predicate
                && first.inner.object == second.inner.object,
        )
    })
}

/// Matches a complete statement against a partial statement whose null fields are wildcards.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_match(
    statement: *mut librdf_statement,
    partial_statement: *mut librdf_statement,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return 0;
        };
        let Some(partial) = (unsafe { borrow_handle(partial_statement, TAG_STATEMENT) }) else {
            return 0;
        };
        i32::from(
            partial
                .inner
                .subject
                .as_ref()
                .is_none_or(|node| statement.inner.subject.as_ref() == Some(node))
                && partial
                    .inner
                    .predicate
                    .as_ref()
                    .is_none_or(|node| statement.inner.predicate.as_ref() == Some(node))
                && partial
                    .inner
                    .object
                    .as_ref()
                    .is_none_or(|node| statement.inner.object.as_ref() == Some(node)),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_is_complete(statement: *mut librdf_statement) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return 0;
        };
        i32::from(
            statement.inner.subject.is_some()
                && statement.inner.predicate.is_some()
                && statement.inner.object.is_some(),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_to_string(statement: *mut librdf_statement) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return ptr::null_mut();
        };
        let s = statement
            .inner
            .subject
            .as_ref()
            .map(|n| n.term.to_string())
            .unwrap_or_else(|| "?".into());
        let p = statement
            .inner
            .predicate
            .as_ref()
            .map(|n| n.term.to_string())
            .unwrap_or_else(|| "?".into());
        let o = statement
            .inner
            .object
            .as_ref()
            .map(|n| n.term.to_string())
            .unwrap_or_else(|| "?".into());
        strdup_c(&format!("{s} {p} {o}")).cast()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_statement_from_statement(
    statement: *mut librdf_statement,
) -> *mut librdf_statement {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return ptr::null_mut();
        };
        box_handle(TAG_STATEMENT, statement.inner.clone())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_statement_from_statement2(
    statement: *mut librdf_statement,
) -> *mut librdf_statement {
    librdf_new_statement_from_statement(statement)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_init(
    _world: *mut librdf_world,
    statement: *mut librdf_statement,
) {
    abort_on_panic(|| {
        clear_last_error();
        if let Some(statement) = unsafe { borrow_handle(statement, TAG_STATEMENT) } {
            statement.inner = StatementInner::default();
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_print(statement: *mut librdf_statement, fh: *mut FILE) {
    abort_on_panic(|| {
        clear_last_error();
        let p = librdf_statement_to_string(statement);
        if p.is_null() {
            return;
        }
        let text = unsafe { std::ffi::CStr::from_ptr(p.cast()) }.to_string_lossy();
        let _ = writeln_file(fh, &text);
        crate::alloc::librdf_free_memory(p.cast());
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_write(
    statement: *mut librdf_statement,
    iostr: *mut c_void,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let p = librdf_statement_to_string(statement);
        if p.is_null() {
            return -1;
        }
        let bytes = unsafe { std::ffi::CStr::from_ptr(p.cast()) }.to_bytes();
        let rc = write_iostream(iostr, bytes);
        crate::alloc::librdf_free_memory(p.cast());
        rc
    })
}

fn encode_node_field(
    node: Option<&crate::handles::node::NodeInner>,
    buffer: *mut u8,
    length: usize,
    offset: &mut usize,
) -> usize {
    let Some(node) = node else {
        // empty marker
        if !buffer.is_null() && length > *offset {
            unsafe { *buffer.add(*offset) = 0 };
        }
        *offset += 1;
        return 1;
    };
    // temporary box to reuse encode
    let tmp = box_handle(TAG_NODE, node.clone());
    let needed = if buffer.is_null() {
        librdf_node_encode(tmp, ptr::null_mut(), 0)
    } else if *offset < length {
        librdf_node_encode(tmp, unsafe { buffer.add(*offset) }, length - *offset)
    } else {
        librdf_node_encode(tmp, ptr::null_mut(), 0)
    };
    unsafe { free_handle(tmp, TAG_NODE) };
    *offset += needed;
    needed
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_encode(
    statement: *mut librdf_statement,
    buffer: *mut u8,
    length: usize,
) -> usize {
    librdf_statement_encode_parts(statement, ptr::null_mut(), buffer, length, 0xff)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_encode2(
    _world: *mut librdf_world,
    statement: *mut librdf_statement,
    buffer: *mut u8,
    length: usize,
) -> usize {
    librdf_statement_encode(statement, buffer, length)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_encode_parts(
    statement: *mut librdf_statement,
    _context_node: *mut librdf_node,
    buffer: *mut u8,
    length: usize,
    fields: u32,
) -> usize {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return 0;
        };
        let mut offset = 0usize;
        let mut total = 0usize;
        if fields & 1 != 0 {
            total += encode_node_field(
                statement.inner.subject.as_ref(),
                buffer,
                length,
                &mut offset,
            );
        }
        if fields & 2 != 0 {
            total += encode_node_field(
                statement.inner.predicate.as_ref(),
                buffer,
                length,
                &mut offset,
            );
        }
        if fields & 4 != 0 {
            total +=
                encode_node_field(statement.inner.object.as_ref(), buffer, length, &mut offset);
        }
        total
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_encode_parts2(
    world: *mut librdf_world,
    statement: *mut librdf_statement,
    context_node: *mut librdf_node,
    buffer: *mut u8,
    length: usize,
    fields: u32,
) -> usize {
    let _ = world;
    librdf_statement_encode_parts(statement, context_node, buffer, length, fields)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_decode(
    statement: *mut librdf_statement,
    buffer: *const u8,
    length: usize,
) -> usize {
    librdf_statement_decode2(ptr::null_mut(), statement, ptr::null_mut(), buffer, length)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_decode_parts(
    statement: *mut librdf_statement,
    context_node: *mut *mut librdf_node,
    buffer: *const u8,
    length: usize,
) -> usize {
    librdf_statement_decode2(ptr::null_mut(), statement, context_node, buffer, length)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_statement_decode2(
    world: *mut librdf_world,
    statement: *mut librdf_statement,
    _context_node: *mut *mut librdf_node,
    buffer: *const u8,
    length: usize,
) -> usize {
    abort_on_panic(|| {
        clear_last_error();
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return 0;
        };
        if buffer.is_null() || length == 0 {
            return 0;
        }
        let mut offset = 0usize;
        let mut nodes = Vec::new();
        for _ in 0..3 {
            if offset >= length {
                break;
            }
            let tmp = box_handle(
                TAG_NODE,
                crate::handles::node::NodeInner::from_term(oxigraph::model::Term::BlankNode(
                    oxigraph::model::BlankNode::default(),
                )),
            );
            let used = crate::handles::node::librdf_node_decode(
                tmp,
                unsafe { buffer.add(offset) },
                length - offset,
            );
            if used == 0 {
                unsafe { free_handle(tmp, TAG_NODE) };
                return 0;
            }
            let inner = unsafe { take_node(tmp) };
            nodes.push(inner);
            offset += used;
        }
        if nodes.len() == 3 {
            statement.inner.subject = nodes[0].clone();
            statement.inner.predicate = nodes[1].clone();
            statement.inner.object = nodes[2].clone();
            statement.inner.clear_cached();
        }
        let _ = world;
        offset
    })
}
