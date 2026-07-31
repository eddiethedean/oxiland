//! `librdf_statement` handle.

use std::ptr;

use oxigraph::model::{NamedOrBlankNode, Term, Triple};

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error};
use crate::handles::node::{NodeInner, librdf_node, take_node};
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_NODE, TAG_STATEMENT, TAG_WORLD, TypedHandle, borrow_handle, box_handle, free_handle,
};

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

    fn clear_cached(&mut self) {
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
