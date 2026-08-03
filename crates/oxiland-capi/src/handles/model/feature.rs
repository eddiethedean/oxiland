//! Feature key/value operations for the C model adapter.

use super::librdf_model;
use crate::error::{abort_on_panic, clear_last_error};
use crate::handles::node::{NodeInner, librdf_node};
use crate::handles::uri::librdf_uri;
use crate::handles::{TAG_MODEL, TAG_NODE, TAG_URI, borrow_handle, box_handle};
use oxigraph::model::Term;
use std::ptr;

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_feature(
    model: *mut librdf_model,
    feature: *mut librdf_uri,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let Some(feature) = (unsafe { borrow_handle(feature, TAG_URI) }) else {
            return ptr::null_mut();
        };
        match model.inner.features.get(feature.inner.node.as_str()) {
            Some(v) => box_handle(
                TAG_NODE,
                NodeInner::from_term(Term::Literal(oxigraph::model::Literal::new_simple_literal(
                    v,
                ))),
            ),
            None => ptr::null_mut(),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_set_feature(
    model: *mut librdf_model,
    feature: *mut librdf_uri,
    value: *mut librdf_node,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        let Some(feature) = (unsafe { borrow_handle(feature, TAG_URI) }) else {
            return -1;
        };
        let Some(value) = (unsafe { borrow_handle(value, TAG_NODE) }) else {
            return -1;
        };
        let text = match &value.inner.term {
            Term::Literal(l) => l.value().to_owned(),
            other => other.to_string(),
        };
        model
            .inner
            .features
            .insert(feature.inner.node.as_str().to_owned(), text);
        0
    })
}
