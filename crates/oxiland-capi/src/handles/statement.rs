//! `librdf_statement` handle.

use std::ptr;

use oxigraph::model::{NamedOrBlankNode, Term, Triple};

use crate::error::{abort_on_panic, clear_last_error};
use crate::handles::node::{NodeInner, librdf_node, take_node};
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_STATEMENT, TAG_WORLD, TypedHandle, borrow_handle, box_handle, free_handle,
};

pub type librdf_statement = TypedHandle<StatementInner>;

#[derive(Clone, Debug, Default)]
pub struct StatementInner {
    pub subject: Option<NodeInner>,
    pub predicate: Option<NodeInner>,
    pub object: Option<NodeInner>,
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
        }
    }
}

/// Creates a statement, taking ownership of the three node handles (Redland semantics).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_statement_from_nodes(
    world: *mut librdf_world,
    subject: *mut librdf_node,
    predicate: *mut librdf_node,
    object: *mut librdf_node,
) -> *mut librdf_statement {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: world is null or a live world handle.
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        // SAFETY: node pointers are null or live node handles; ownership transfers.
        let subject = unsafe { take_node(subject) };
        let predicate = unsafe { take_node(predicate) };
        let object = unsafe { take_node(object) };
        box_handle(
            TAG_STATEMENT,
            StatementInner {
                subject,
                predicate,
                object,
            },
        )
    })
}

/// Frees a statement (and owned nodes). Null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_statement(statement: *mut librdf_statement) {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: statement is null or a live statement handle.
        unsafe { free_handle(statement, TAG_STATEMENT) };
    });
}
