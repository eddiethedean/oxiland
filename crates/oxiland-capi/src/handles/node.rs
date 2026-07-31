//! `librdf_node` handle.

use std::os::raw::c_char;
use std::ptr;

use oxigraph::model::{
    BlankNode, Literal, NamedNode, NamedNodeRef, NamedOrBlankNode, NamedOrBlankNodeRef, Term,
    TermRef,
};

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::uri::{UriInner, librdf_uri};
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_NODE, TAG_URI, TAG_WORLD, TypedHandle, borrow_handle, box_handle, cstr_optional,
    cstr_required, free_handle,
};

pub type librdf_node = TypedHandle<NodeInner>;

#[derive(Debug)]
pub struct NodeInner {
    pub term: Term,
    literal_c: Option<*mut c_char>,
    language_c: Option<*mut c_char>,
    blank_c: Option<*mut c_char>,
    uri_cache: Option<*mut librdf_uri>,
}

impl Drop for NodeInner {
    fn drop(&mut self) {
        for ptr in [
            self.literal_c.take(),
            self.language_c.take(),
            self.blank_c.take(),
        ]
        .into_iter()
        .flatten()
        {
            if !ptr.is_null() {
                unsafe { libc::free(ptr.cast()) };
            }
        }
        if let Some(ptr) = self.uri_cache.take() {
            if !ptr.is_null() {
                unsafe { free_handle(ptr, TAG_URI) };
            }
        }
    }
}

impl PartialEq for NodeInner {
    fn eq(&self, other: &Self) -> bool {
        self.term == other.term
    }
}

impl Eq for NodeInner {}

impl Clone for NodeInner {
    fn clone(&self) -> Self {
        Self::from_term(self.term.clone())
    }
}

impl NodeInner {
    pub fn from_term(term: Term) -> Self {
        Self {
            term,
            literal_c: None,
            language_c: None,
            blank_c: None,
            uri_cache: None,
        }
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

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node_from_uri_string(
    world: *mut librdf_world,
    uri_string: *const c_char,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
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

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node_from_literal(
    world: *mut librdf_world,
    string: *const c_char,
    xml_language: *const c_char,
    _is_wf_xml: i32,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        let Some(string) = (unsafe { cstr_required(string, "string") }) else {
            return ptr::null_mut();
        };
        let language = match unsafe { cstr_optional(xml_language, "xml_language") } {
            Ok(v) => v,
            Err(()) => return ptr::null_mut(),
        };
        let literal = match language {
            Some(lang) if !lang.is_empty() => {
                match Literal::new_language_tagged_literal(string, lang) {
                    Ok(literal) => literal,
                    Err(error) => {
                        set_last_error(error.to_string());
                        return ptr::null_mut();
                    }
                }
            }
            _ => Literal::new_simple_literal(string),
        };
        box_handle(TAG_NODE, NodeInner::from_term(Term::Literal(literal)))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node_from_blank_identifier(
    world: *mut librdf_world,
    identifier: *const u8,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        let id = match unsafe { cstr_optional(identifier.cast(), "identifier") } {
            Ok(v) => v,
            Err(()) => return ptr::null_mut(),
        };
        let blank = match id {
            Some(id) if !id.is_empty() => match BlankNode::new(id) {
                Ok(b) => b,
                Err(error) => {
                    set_last_error(error.to_string());
                    return ptr::null_mut();
                }
            },
            _ => BlankNode::default(),
        };
        box_handle(TAG_NODE, NodeInner::from_term(Term::BlankNode(blank)))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_node(node: *mut librdf_node) {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { free_handle(node, TAG_NODE) };
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_get_type(node: *mut librdf_node) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return 0;
        };
        match &node.inner.term {
            Term::NamedNode(_) => 1,
            Term::Literal(_) => 2,
            Term::BlankNode(_) => 4,
            #[allow(unreachable_patterns)]
            _ => 0,
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_is_resource(node: *mut librdf_node) -> i32 {
    i32::from(librdf_node_get_type(node) == 1)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_is_literal(node: *mut librdf_node) -> i32 {
    i32::from(librdf_node_get_type(node) == 2)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_is_blank(node: *mut librdf_node) -> i32 {
    i32::from(librdf_node_get_type(node) == 4)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_get_uri(node: *mut librdf_node) -> *mut librdf_uri {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return ptr::null_mut();
        };
        let Term::NamedNode(named) = &node.inner.term else {
            return ptr::null_mut();
        };
        if let Some(ptr) = node.inner.uri_cache {
            return ptr;
        }
        let ptr = box_handle(TAG_URI, UriInner::new(named.clone()));
        node.inner.uri_cache = Some(ptr);
        ptr
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_get_literal_value(node: *mut librdf_node) -> *const c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return ptr::null();
        };
        let Term::Literal(lit) = &node.inner.term else {
            return ptr::null();
        };
        if node.inner.literal_c.is_none() {
            node.inner.literal_c = Some(strdup_c(lit.value()));
        }
        node.inner.literal_c.unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_get_literal_value_language(node: *mut librdf_node) -> *const c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return ptr::null();
        };
        let Term::Literal(lit) = &node.inner.term else {
            return ptr::null();
        };
        let Some(lang) = lit.language() else {
            return ptr::null();
        };
        if node.inner.language_c.is_none() {
            node.inner.language_c = Some(strdup_c(lang));
        }
        node.inner.language_c.unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_get_blank_identifier(node: *mut librdf_node) -> *const c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return ptr::null();
        };
        let Term::BlankNode(blank) = &node.inner.term else {
            return ptr::null();
        };
        if node.inner.blank_c.is_none() {
            node.inner.blank_c = Some(strdup_c(blank.as_str()));
        }
        node.inner.blank_c.unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_to_string(node: *mut librdf_node) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return ptr::null_mut();
        };
        strdup_c(&node.inner.term.to_string()).cast()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_equals(first: *mut librdf_node, second: *mut librdf_node) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(first) = (unsafe { borrow_handle(first, TAG_NODE) }) else {
            return 0;
        };
        let Some(second) = (unsafe { borrow_handle(second, TAG_NODE) }) else {
            return 0;
        };
        i32::from(first.inner.term == second.inner.term)
    })
}

/// Takes ownership of a node pointer into a [`NodeInner`], unregistering it.
///
/// # Safety
/// `ptr` must be null or a live node handle from this crate.
pub unsafe fn take_node(ptr: *mut librdf_node) -> Option<NodeInner> {
    if ptr.is_null() {
        return None;
    }
    let handle = unsafe { crate::handles::borrow_handle(ptr, TAG_NODE)? };
    let inner = NodeInner::from_term(handle.inner.term.clone());
    unsafe { free_handle(ptr, TAG_NODE) };
    Some(inner)
}
