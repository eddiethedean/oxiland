//! RDF / RDFS concept URI and node helpers.

use std::ptr;
use std::sync::OnceLock;

use oxigraph::model::NamedNode;

use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::node::{NodeInner, librdf_node};
use crate::handles::uri::{UriInner, librdf_uri};
use crate::handles::world::librdf_world;
use crate::handles::{TAG_NODE, TAG_URI, TAG_WORLD, borrow_handle, box_handle};

const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const RDFS_NS: &str = "http://www.w3.org/2000/01/rdf-schema#";

static CONCEPTS: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();

fn concepts() -> &'static [(&'static str, &'static str)] {
    CONCEPTS.get_or_init(|| {
        vec![
            (RDF_NS, "Alt"),
            (RDF_NS, "Bag"),
            (RDF_NS, "Property"),
            (RDF_NS, "Seq"),
            (RDF_NS, "Statement"),
            (RDF_NS, "object"),
            (RDF_NS, "predicate"),
            (RDF_NS, "subject"),
            (RDF_NS, "type"),
            (RDF_NS, "value"),
            (RDF_NS, "li"),
            (RDF_NS, "RDF"),
            (RDF_NS, "Description"),
            (RDF_NS, "aboutEach"),
            (RDF_NS, "aboutEachPrefix"),
            (RDF_NS, "nodeID"),
            (RDF_NS, "List"),
            (RDF_NS, "first"),
            (RDF_NS, "rest"),
            (RDF_NS, "nil"),
            (RDF_NS, "XMLLiteral"),
            (RDFS_NS, "Class"),
            (RDFS_NS, "ConstraintProperty"),
            (RDFS_NS, "ConstraintResource"),
            (RDFS_NS, "Container"),
            (RDFS_NS, "ContainerMembershipProperty"),
            (RDFS_NS, "Literal"),
            (RDFS_NS, "Resource"),
            (RDFS_NS, "comment"),
            (RDFS_NS, "domain"),
            (RDFS_NS, "isDefinedBy"),
            (RDFS_NS, "label"),
            (RDFS_NS, "range"),
            (RDFS_NS, "seeAlso"),
            (RDFS_NS, "subClassOf"),
            (RDFS_NS, "subPropertyOf"),
            (RDF_NS, "HTML"),
            (RDF_NS, "langString"),
        ]
    })
}

fn concept_iri(idx: u32) -> Option<String> {
    concepts()
        .get(idx as usize)
        .map(|(ns, local)| format!("{ns}{local}"))
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_get_concept_ms_namespace(world: *mut librdf_world) -> *mut librdf_uri {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        match NamedNode::new(RDF_NS) {
            Ok(n) => box_handle(TAG_URI, UriInner::new(n)),
            Err(e) => {
                set_last_error(e.to_string());
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_get_concept_schema_namespace(world: *mut librdf_world) -> *mut librdf_uri {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        match NamedNode::new(RDFS_NS) {
            Ok(n) => box_handle(TAG_URI, UriInner::new(n)),
            Err(e) => {
                set_last_error(e.to_string());
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_get_concept_uri_by_index(
    world: *mut librdf_world,
    idx: u32,
) -> *mut librdf_uri {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        let Some(iri) = concept_iri(idx) else {
            set_last_error("concept index out of range");
            return ptr::null_mut();
        };
        match NamedNode::new(iri) {
            Ok(n) => box_handle(TAG_URI, UriInner::new(n)),
            Err(e) => {
                set_last_error(e.to_string());
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_get_concept_resource_by_index(
    world: *mut librdf_world,
    idx: u32,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        let Some(iri) = concept_iri(idx) else {
            set_last_error("concept index out of range");
            return ptr::null_mut();
        };
        match NamedNode::new(iri) {
            Ok(n) => box_handle(
                TAG_NODE,
                NodeInner::from_term(oxigraph::model::Term::NamedNode(n)),
            ),
            Err(e) => {
                set_last_error(e.to_string());
                ptr::null_mut()
            }
        }
    })
}
