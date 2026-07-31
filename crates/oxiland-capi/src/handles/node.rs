//! `librdf_node` handle.

use std::ptr;

use oxigraph::model::{
    Literal, NamedNode, NamedNodeRef, NamedOrBlankNode, NamedOrBlankNodeRef, Term, TermRef,
};

use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_NODE, TAG_WORLD, TypedHandle, borrow_handle, box_handle, cstr_optional, cstr_required,
    free_handle,
};

pub type librdf_node = TypedHandle<NodeInner>;

#[derive(Clone, Debug)]
pub struct NodeInner {
    pub term: Term,
}

impl NodeInner {
    pub fn from_term(term: Term) -> Self {
        Self { term }
    }

    pub fn as_named(&self) -> Option<NamedNode> {
        match &self.term {
            Term::NamedNode(n) => Some(n.clone()),
            _ => None,
        }
    }

    pub fn as_named_ref(&self) -> Option<NamedNodeRef<'_>> {
        match &self.term {
            Term::NamedNode(n) => Some(n.as_ref()),
            _ => None,
        }
    }

    pub fn as_named_or_blank(&self) -> Option<NamedOrBlankNode> {
        match &self.term {
            Term::NamedNode(n) => Some(NamedOrBlankNode::NamedNode(n.clone())),
            Term::BlankNode(b) => Some(NamedOrBlankNode::BlankNode(b.clone())),
            Term::Literal(_) => None,
            #[allow(unreachable_patterns)]
            _ => None,
        }
    }

    pub fn as_named_or_blank_ref(&self) -> Option<NamedOrBlankNodeRef<'_>> {
        match &self.term {
            Term::NamedNode(n) => Some(NamedOrBlankNodeRef::NamedNode(n.as_ref())),
            Term::BlankNode(b) => Some(NamedOrBlankNodeRef::BlankNode(b.as_ref())),
            Term::Literal(_) => None,
            #[allow(unreachable_patterns)]
            _ => None,
        }
    }

    pub fn as_term_ref(&self) -> TermRef<'_> {
        TermRef::from(&self.term)
    }
}

/// Creates an IRI node from a URI string.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node_from_uri_string(
    world: *mut librdf_world,
    uri_string: *const std::os::raw::c_char,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: world is null or a live world handle.
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        // SAFETY: uri_string is a C string when non-null.
        let Some(uri_string) = (unsafe { cstr_required(uri_string, "uri_string") }) else {
            return ptr::null_mut();
        };
        match NamedNode::new(uri_string) {
            Ok(node) => box_handle(TAG_NODE, NodeInner::from_term(Term::NamedNode(node))),
            Err(error) => {
                set_last_error(error.to_string());
                ptr::null_mut()
            }
        }
    })
}

/// Creates a literal node. `is_wf_xml` is ignored in the preview.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node_from_literal(
    world: *mut librdf_world,
    string: *const std::os::raw::c_char,
    xml_language: *const std::os::raw::c_char,
    _is_wf_xml: i32,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: world is null or a live world handle.
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        // SAFETY: string is a C string when non-null.
        let Some(string) = (unsafe { cstr_required(string, "string") }) else {
            return ptr::null_mut();
        };
        // SAFETY: xml_language is optional C string.
        let language = match unsafe { cstr_optional(xml_language, "xml_language") } {
            Ok(v) => v,
            Err(()) => return ptr::null_mut(),
        };
        let literal = match language {
            Some(lang) if !lang.is_empty() => Literal::new_language_tagged_literal(string, lang)
                .unwrap_or_else(|_| Literal::new_simple_literal(string)),
            _ => Literal::new_simple_literal(string),
        };
        box_handle(TAG_NODE, NodeInner::from_term(Term::Literal(literal)))
    })
}

/// Frees a node. Null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_node(node: *mut librdf_node) {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: node is null or a live node handle.
        unsafe { free_handle(node, TAG_NODE) };
    });
}

/// Takes ownership of a node pointer into a [`NodeInner`], unregistering it.
///
/// # Safety
/// `ptr` must be null or a live node handle from this crate.
pub unsafe fn take_node(ptr: *mut librdf_node) -> Option<NodeInner> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: live node handle; free_handle path without drop of wrong type.
    let handle = unsafe { crate::handles::borrow_handle(ptr, TAG_NODE)? };
    let inner = handle.inner.clone();
    // SAFETY: transfer ownership out of the C handle.
    unsafe { free_handle(ptr, TAG_NODE) };
    Some(inner)
}
