//! Redland graph-navigation operations for `librdf_model`.
//!
//! This module owns the legacy source/arc/target projection API. Keeping it
//! separate from model lifecycle and mutation makes the projection rules and
//! iterator ownership independently testable.

use super::{ModelInner, librdf_model};
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::iterator::{box_items, librdf_iterator};
use crate::handles::node::{NodeInner, librdf_node};
use crate::handles::{TAG_MODEL, TAG_NODE, borrow_handle, box_handle};
use oxigraph::model::{GraphName, Quad, Term};
use oxiland::{Model, StatementPattern};
use std::collections::HashSet;
use std::ffi::c_void;
use std::ptr;

#[derive(Clone, Copy)]
enum Projection {
    Subject,
    Predicate,
    Object,
}

impl Projection {
    fn apply(self, quad: &Quad) -> Term {
        match self {
            Self::Subject => Term::from(quad.subject.clone()),
            Self::Predicate => Term::from(quad.predicate.clone()),
            Self::Object => quad.object.clone(),
        }
    }
}

fn node_term(node: *mut librdf_node) -> Option<Term> {
    let node = unsafe { borrow_handle(node, TAG_NODE) }?;
    Some(node.inner.term.clone())
}

fn collect_matching_nodes(
    model: &Model,
    subject: Option<&Term>,
    predicate: Option<&Term>,
    object: Option<&Term>,
    projection: Projection,
) -> Result<Vec<*mut c_void>, String> {
    let mut items = Vec::new();
    let mut seen = HashSet::new();
    for item in model.find(StatementPattern::default()) {
        let quad = item.map_err(|error| error.to_string())?;
        let quad_subject = Term::from(quad.subject.clone());
        let quad_predicate = Term::from(quad.predicate.clone());
        if subject.is_some_and(|term| term != &quad_subject)
            || predicate.is_some_and(|term| term != &quad_predicate)
            || object.is_some_and(|term| term != &quad.object)
        {
            continue;
        }
        let term = projection.apply(&quad);
        if seen.insert(term.clone()) {
            items.push(box_handle(TAG_NODE, NodeInner::from_term(term)).cast());
        }
    }
    Ok(items)
}

fn projected_iterator(
    model: &ModelInner,
    subject: Option<&Term>,
    predicate: Option<&Term>,
    object: Option<&Term>,
    projection: Projection,
) -> *mut librdf_iterator {
    match collect_matching_nodes(&model.model, subject, predicate, object, projection) {
        Ok(items) => box_items(items),
        Err(error) => {
            set_last_error(error);
            ptr::null_mut()
        }
    }
}

fn first_owned_node(iterator: *mut librdf_iterator) -> *mut librdf_node {
    if iterator.is_null() {
        return ptr::null_mut();
    }
    let object = crate::handles::iterator::librdf_iterator_get_object(iterator);
    let node = unsafe { borrow_handle(object.cast::<librdf_node>(), TAG_NODE) }
        .map(|node| box_handle(TAG_NODE, NodeInner::from_term(node.inner.term.clone())))
        .unwrap_or(ptr::null_mut());
    crate::handles::iterator::librdf_free_iterator(iterator);
    node
}

fn iterator_contains(iterator: *mut librdf_iterator, wanted: Option<Term>) -> i32 {
    if iterator.is_null() {
        return 0;
    }
    let mut found = false;
    while crate::handles::iterator::librdf_iterator_end(iterator) == 0 {
        let object = crate::handles::iterator::librdf_iterator_get_object(iterator);
        let current = unsafe { borrow_handle(object.cast::<librdf_node>(), TAG_NODE) }
            .map(|node| node.inner.term.clone());
        if wanted
            .as_ref()
            .zip(current.as_ref())
            .is_some_and(|(a, b)| a == b)
        {
            found = true;
            break;
        }
        if crate::handles::iterator::librdf_iterator_next(iterator) != 0 {
            break;
        }
    }
    crate::handles::iterator::librdf_free_iterator(iterator);
    i32::from(found)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_sources(
    model: *mut librdf_model,
    arc: *mut librdf_node,
    target: *mut librdf_node,
) -> *mut librdf_iterator {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let predicate = node_term(arc);
        let object = node_term(target);
        projected_iterator(
            &model.inner,
            None,
            predicate.as_ref(),
            object.as_ref(),
            Projection::Subject,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_arcs(
    model: *mut librdf_model,
    source: *mut librdf_node,
    target: *mut librdf_node,
) -> *mut librdf_iterator {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let subject = node_term(source);
        let object = node_term(target);
        projected_iterator(
            &model.inner,
            subject.as_ref(),
            None,
            object.as_ref(),
            Projection::Predicate,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_targets(
    model: *mut librdf_model,
    source: *mut librdf_node,
    arc: *mut librdf_node,
) -> *mut librdf_iterator {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let subject = node_term(source);
        let predicate = node_term(arc);
        projected_iterator(
            &model.inner,
            subject.as_ref(),
            predicate.as_ref(),
            None,
            Projection::Object,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_source(
    model: *mut librdf_model,
    arc: *mut librdf_node,
    target: *mut librdf_node,
) -> *mut librdf_node {
    first_owned_node(librdf_model_get_sources(model, arc, target))
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_arc(
    model: *mut librdf_model,
    source: *mut librdf_node,
    target: *mut librdf_node,
) -> *mut librdf_node {
    first_owned_node(librdf_model_get_arcs(model, source, target))
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_target(
    model: *mut librdf_model,
    source: *mut librdf_node,
    arc: *mut librdf_node,
) -> *mut librdf_node {
    first_owned_node(librdf_model_get_targets(model, source, arc))
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_arcs_in(
    model: *mut librdf_model,
    node: *mut librdf_node,
) -> *mut librdf_iterator {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let object = node_term(node);
        projected_iterator(
            &model.inner,
            None,
            None,
            object.as_ref(),
            Projection::Predicate,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_arcs_out(
    model: *mut librdf_model,
    node: *mut librdf_node,
) -> *mut librdf_iterator {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let subject = node_term(node);
        projected_iterator(
            &model.inner,
            subject.as_ref(),
            None,
            None,
            Projection::Predicate,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_has_arc_in(
    model: *mut librdf_model,
    node: *mut librdf_node,
    property: *mut librdf_node,
) -> i32 {
    let wanted = node_term(property);
    iterator_contains(librdf_model_get_arcs_in(model, node), wanted)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_has_arc_out(
    model: *mut librdf_model,
    node: *mut librdf_node,
    property: *mut librdf_node,
) -> i32 {
    let wanted = node_term(property);
    iterator_contains(librdf_model_get_arcs_out(model, node), wanted)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_contexts(model: *mut librdf_model) -> *mut librdf_iterator {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let mut seen = HashSet::new();
        let mut items = Vec::new();
        for item in model.inner.model.find(StatementPattern::default()) {
            match item {
                Ok(quad) => {
                    if let GraphName::NamedNode(node) = quad.graph_name {
                        if seen.insert(node.clone()) {
                            items.push(
                                box_handle(TAG_NODE, NodeInner::from_term(Term::NamedNode(node)))
                                    .cast(),
                            );
                        }
                    }
                }
                Err(error) => {
                    set_last_error(error.to_string());
                    return ptr::null_mut();
                }
            }
        }
        box_items(items)
    })
}
