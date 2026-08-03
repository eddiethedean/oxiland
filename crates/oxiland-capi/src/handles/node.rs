//! `librdf_node` handle.

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::io::{FILE, write_iostream, writeln_file};
use crate::handles::iterator::{box_items, librdf_iterator};
use crate::handles::uri::{UriInner, librdf_uri};
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_NODE, TAG_URI, TAG_WORLD, TypedHandle, borrow_handle, box_handle, cstr_optional,
    cstr_required, free_handle,
};
use oxigraph::model::{
    BlankNode, Literal, NamedNode, NamedNodeRef, NamedOrBlankNode, NamedOrBlankNodeRef, Term,
    TermRef,
};
use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr;

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
            #[allow(
                unreachable_patterns,
                reason = "keep the adapter forward-compatible with non-exhaustive engine terms"
            )]
            _ => None,
        }
    }

    pub fn as_named_or_blank_ref(&self) -> Option<NamedOrBlankNodeRef<'_>> {
        match &self.term {
            Term::NamedNode(n) => Some(NamedOrBlankNodeRef::NamedNode(n.as_ref())),
            Term::BlankNode(b) => Some(NamedOrBlankNodeRef::BlankNode(b.as_ref())),
            Term::Literal(_) => None,
            #[allow(
                unreachable_patterns,
                reason = "keep the adapter forward-compatible with non-exhaustive engine terms"
            )]
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
            #[allow(
                unreachable_patterns,
                reason = "keep the adapter forward-compatible with non-exhaustive engine terms"
            )]
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

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node(world: *mut librdf_world) -> *mut librdf_node {
    // Redland creates an empty/unknown node; we use a fresh blank.
    librdf_new_node_from_blank_identifier(world, ptr::null())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node_from_uri(
    world: *mut librdf_world,
    uri: *mut librdf_uri,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        let Some(uri) = (unsafe { borrow_handle(uri, TAG_URI) }) else {
            return ptr::null_mut();
        };
        box_handle(
            TAG_NODE,
            NodeInner::from_term(Term::NamedNode(uri.inner.node.clone())),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node_from_uri_local_name(
    world: *mut librdf_world,
    uri: *mut librdf_uri,
    local_name: *const c_char,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        let Some(uri) = (unsafe { borrow_handle(uri, TAG_URI) }) else {
            return ptr::null_mut();
        };
        let Some(local) = (unsafe { cstr_required(local_name, "local_name") }) else {
            return ptr::null_mut();
        };
        let iri = format!("{}{local}", uri.inner.node.as_str());
        match NamedNode::new(iri) {
            Ok(n) => box_handle(TAG_NODE, NodeInner::from_term(Term::NamedNode(n))),
            Err(e) => {
                set_last_error(e.to_string());
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node_from_normalised_uri_string(
    world: *mut librdf_world,
    uri_string: *const c_char,
    source_uri: *mut librdf_uri,
    base_uri: *mut librdf_uri,
) -> *mut librdf_node {
    let _ = (source_uri, base_uri);
    librdf_new_node_from_uri_string(world, uri_string)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node_from_node(node: *mut librdf_node) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return ptr::null_mut();
        };
        box_handle(TAG_NODE, node.inner.clone())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node_from_counted_uri_string(
    world: *mut librdf_world,
    uri_string: *const c_char,
    length: usize,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        if uri_string.is_null() {
            set_last_error("uri_string is null");
            return ptr::null_mut();
        }
        let bytes = unsafe { std::slice::from_raw_parts(uri_string.cast::<u8>(), length) };
        let s = match std::str::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => {
                set_last_error("uri_string is not UTF-8");
                return ptr::null_mut();
            }
        };
        let c = match std::ffi::CString::new(s) {
            Ok(c) => c,
            Err(_) => {
                set_last_error("uri_string contains NUL");
                return ptr::null_mut();
            }
        };
        librdf_new_node_from_uri_string(world, c.as_ptr())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node_from_counted_blank_identifier(
    world: *mut librdf_world,
    identifier: *const u8,
    length: usize,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        if identifier.is_null() {
            return librdf_new_node_from_blank_identifier(world, ptr::null());
        }
        let bytes = unsafe { std::slice::from_raw_parts(identifier, length) };
        let s = match std::str::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => {
                set_last_error("identifier is not UTF-8");
                return ptr::null_mut();
            }
        };
        let c = match std::ffi::CString::new(s) {
            Ok(c) => c,
            Err(_) => {
                set_last_error("identifier contains NUL");
                return ptr::null_mut();
            }
        };
        librdf_new_node_from_blank_identifier(world, c.as_ptr().cast())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node_from_typed_literal(
    world: *mut librdf_world,
    value: *const c_char,
    xml_language: *const c_char,
    datatype_uri: *mut librdf_uri,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        let Some(value) = (unsafe { cstr_required(value, "value") }) else {
            return ptr::null_mut();
        };
        let language = match unsafe { cstr_optional(xml_language, "xml_language") } {
            Ok(v) => v,
            Err(()) => return ptr::null_mut(),
        };
        if language.is_some_and(|l| !l.is_empty()) && !datatype_uri.is_null() {
            set_last_error("typed literal cannot have language");
            return ptr::null_mut();
        }
        let literal = if !datatype_uri.is_null() {
            let Some(dt) = (unsafe { borrow_handle(datatype_uri, TAG_URI) }) else {
                return ptr::null_mut();
            };
            Literal::new_typed_literal(value, dt.inner.node.clone())
        } else if let Some(lang) = language.filter(|l| !l.is_empty()) {
            match Literal::new_language_tagged_literal(value, lang) {
                Ok(l) => l,
                Err(e) => {
                    set_last_error(e.to_string());
                    return ptr::null_mut();
                }
            }
        } else {
            Literal::new_simple_literal(value)
        };
        box_handle(TAG_NODE, NodeInner::from_term(Term::Literal(literal)))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_node_from_typed_counted_literal(
    world: *mut librdf_world,
    value: *const c_char,
    value_len: usize,
    xml_language: *const c_char,
    xml_language_len: usize,
    datatype_uri: *mut librdf_uri,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        if value.is_null() {
            set_last_error("value is null");
            return ptr::null_mut();
        }
        let vbytes = unsafe { std::slice::from_raw_parts(value.cast::<u8>(), value_len) };
        let v = match std::str::from_utf8(vbytes) {
            Ok(s) => s,
            Err(_) => {
                set_last_error("value is not UTF-8");
                return ptr::null_mut();
            }
        };
        let lang_c = if xml_language.is_null() || xml_language_len == 0 {
            None
        } else {
            let lbytes =
                unsafe { std::slice::from_raw_parts(xml_language.cast::<u8>(), xml_language_len) };
            match std::str::from_utf8(lbytes) {
                Ok(s) => Some(s.to_owned()),
                Err(_) => {
                    set_last_error("language is not UTF-8");
                    return ptr::null_mut();
                }
            }
        };
        let vc = match std::ffi::CString::new(v) {
            Ok(c) => c,
            Err(_) => {
                set_last_error("value contains NUL");
                return ptr::null_mut();
            }
        };
        let lc = lang_c
            .as_ref()
            .and_then(|l| std::ffi::CString::new(l.as_str()).ok());
        librdf_new_node_from_typed_literal(
            world,
            vc.as_ptr(),
            lc.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null()),
            datatype_uri,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_get_li_ordinal(node: *mut librdf_node) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return -1;
        };
        let Term::NamedNode(n) = &node.inner.term else {
            return -1;
        };
        let s = n.as_str();
        const PREFIX: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#_";
        if let Some(rest) = s.strip_prefix(PREFIX) {
            rest.parse().unwrap_or(-1)
        } else {
            -1
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_get_literal_value_as_counted_string(
    node: *mut librdf_node,
    len_p: *mut usize,
) -> *const c_char {
    let p = librdf_node_get_literal_value(node);
    if !p.is_null() && !len_p.is_null() {
        let s = unsafe { std::ffi::CStr::from_ptr(p) };
        unsafe { *len_p = s.to_bytes().len() };
    }
    p
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_get_literal_value_as_latin1(node: *mut librdf_node) -> *mut c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return ptr::null_mut();
        };
        let Term::Literal(lit) = &node.inner.term else {
            return ptr::null_mut();
        };
        let mut out = Vec::new();
        for ch in lit.value().chars() {
            out.push(if (ch as u32) <= 0xff { ch as u8 } else { b'?' });
        }
        out.push(0);
        let ptr = unsafe { libc::malloc(out.len()) }.cast::<c_char>();
        if ptr.is_null() {
            set_last_error("out of memory");
            return ptr::null_mut();
        }
        unsafe { ptr::copy_nonoverlapping(out.as_ptr().cast(), ptr, out.len()) };
        ptr
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_get_literal_value_datatype_uri(
    node: *mut librdf_node,
) -> *mut librdf_uri {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return ptr::null_mut();
        };
        let Term::Literal(lit) = &node.inner.term else {
            return ptr::null_mut();
        };
        // Always has a datatype in RDF 1.1 (xsd:string / rdf:langString / explicit)
        box_handle(TAG_URI, UriInner::new(lit.datatype().into_owned()))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_get_literal_value_is_wf_xml(node: *mut librdf_node) -> i32 {
    // Oxiland does not track wf-xml; return 0.
    let _ = node;
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_get_counted_blank_identifier(
    node: *mut librdf_node,
    len_p: *mut usize,
) -> *const c_char {
    let p = librdf_node_get_blank_identifier(node);
    if !p.is_null() && !len_p.is_null() {
        let s = unsafe { std::ffi::CStr::from_ptr(p) };
        unsafe { *len_p = s.to_bytes().len() };
    }
    p
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_to_counted_string(
    node: *mut librdf_node,
    len_p: *mut usize,
) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return ptr::null_mut();
        };
        let text = node.inner.term.to_string();
        if !len_p.is_null() {
            unsafe { *len_p = text.len() };
        }
        strdup_c(&text).cast()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_print(node: *mut librdf_node, fh: *mut FILE) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return;
        };
        let _ = writeln_file(fh, &node.inner.term.to_string());
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_write(node: *mut librdf_node, iostr: *mut c_void) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return -1;
        };
        write_iostream(iostr, node.inner.term.to_string().as_bytes())
    })
}

/// Encode node as length-prefixed UTF-8 (Oxiland portable encoding).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_encode(
    node: *mut librdf_node,
    buffer: *mut u8,
    length: usize,
) -> usize {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return 0;
        };
        let text = node.inner.term.to_string();
        let needed = text.len() + 1;
        if buffer.is_null() || length < needed {
            return needed;
        }
        unsafe {
            ptr::copy_nonoverlapping(text.as_ptr(), buffer, text.len());
            *buffer.add(text.len()) = 0;
        }
        needed
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_decode(
    node: *mut librdf_node,
    buffer: *const u8,
    length: usize,
) -> usize {
    abort_on_panic(|| {
        clear_last_error();
        let Some(node) = (unsafe { borrow_handle(node, TAG_NODE) }) else {
            return 0;
        };
        if buffer.is_null() || length == 0 {
            return 0;
        }
        let bytes = unsafe { std::slice::from_raw_parts(buffer, length) };
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let text = match std::str::from_utf8(&bytes[..end]) {
            Ok(t) => t,
            Err(_) => {
                set_last_error("decode buffer not UTF-8");
                return 0;
            }
        };
        // Best-effort: IRI, blank, or literal string.
        let term = if let Some(id) = text.strip_prefix("_:") {
            BlankNode::new(id).ok().map(Term::BlankNode)
        } else if text.starts_with('<') && text.ends_with('>') {
            NamedNode::new(&text[1..text.len() - 1])
                .ok()
                .map(Term::NamedNode)
        } else if text.starts_with('"') {
            Some(Term::Literal(Literal::new_simple_literal(
                text.trim_matches('"'),
            )))
        } else {
            NamedNode::new(text).ok().map(Term::NamedNode)
        };
        match term {
            Some(t) => {
                node.inner = NodeInner::from_term(t);
                end + if end < bytes.len() { 1 } else { 0 }
            }
            None => 0,
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_static_iterator_create(
    nodes: *mut *mut librdf_node,
    length: i32,
) -> *mut librdf_iterator {
    abort_on_panic(|| {
        clear_last_error();
        if nodes.is_null() || length < 0 {
            return ptr::null_mut();
        }
        let len = length as usize;
        let mut items = Vec::with_capacity(len);
        for i in 0..len {
            let p = unsafe { *nodes.add(i) };
            items.push(p.cast());
        }
        box_items(items)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_node_new_static_node_iterator(
    world: *mut librdf_world,
    nodes: *mut *mut librdf_node,
    length: i32,
) -> *mut librdf_iterator {
    let _ = world;
    librdf_node_static_iterator_create(nodes, length)
}
