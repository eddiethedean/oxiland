//! Named-graph/context operations for the C model adapter.

use super::{context_graph, librdf_model, statement_as_triple, stream_for_pattern};
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::node::librdf_node;
use crate::handles::statement::{StatementInner, librdf_statement};
use crate::handles::stream::{
    librdf_stream, librdf_stream_end, librdf_stream_get_object, librdf_stream_next,
};
use crate::handles::{TAG_MODEL, TAG_NODE, TAG_STATEMENT, TAG_STREAM, borrow_handle};
use oxiland::StatementPattern;
use std::ptr;

/// Adds a statement to a named graph/context.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_context_add_statement(
    model: *mut librdf_model,
    context: *mut librdf_node,
    statement: *mut librdf_statement,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        let Some(context) = (unsafe { borrow_handle(context, crate::handles::TAG_NODE) }) else {
            return -1;
        };
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return -1;
        };
        let graph = match context_graph(&context.inner) {
            Ok(graph) => graph,
            Err(error) => {
                set_last_error(error);
                return -1;
            }
        };
        let triple = match statement_as_triple(&statement.inner) {
            Ok(triple) => triple,
            Err(error) => {
                set_last_error(error);
                return -1;
            }
        };
        model.inner.cardinality.invalidate();
        match model.inner.model.add_to_graph(triple, graph) {
            Ok(_) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

/// Removes a statement from a named graph/context.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_context_remove_statement(
    model: *mut librdf_model,
    context: *mut librdf_node,
    statement: *mut librdf_statement,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        let Some(context) = (unsafe { borrow_handle(context, crate::handles::TAG_NODE) }) else {
            return -1;
        };
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return -1;
        };
        let graph = match context_graph(&context.inner) {
            Ok(graph) => graph,
            Err(error) => {
                set_last_error(error);
                return -1;
            }
        };
        let triple = match statement_as_triple(&statement.inner) {
            Ok(triple) => triple,
            Err(error) => {
                set_last_error(error);
                return -1;
            }
        };
        model.inner.cardinality.invalidate();
        match model.inner.model.remove_from_graph(triple, graph) {
            Ok(_) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

/// Returns statements from a named graph/context.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_context_as_stream(
    model: *mut librdf_model,
    context: *mut librdf_node,
) -> *mut librdf_stream {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let Some(context) = (unsafe { borrow_handle(context, crate::handles::TAG_NODE) }) else {
            return ptr::null_mut();
        };
        let graph = match context_graph(&context.inner) {
            Ok(graph) => graph,
            Err(error) => {
                set_last_error(error);
                return ptr::null_mut();
            }
        };
        match stream_for_pattern(&model.inner.model, &StatementInner::default(), Some(graph)) {
            Ok(stream) => stream,
            Err(error) => {
                set_last_error(error);
                ptr::null_mut()
            }
        }
    })
}

/// Finds statements matching a pattern in a named graph/context.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_find_statements_in_context(
    model: *mut librdf_model,
    statement: *mut librdf_statement,
    context: *mut librdf_node,
) -> *mut librdf_stream {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return ptr::null_mut();
        };
        let Some(context) = (unsafe { borrow_handle(context, crate::handles::TAG_NODE) }) else {
            return ptr::null_mut();
        };
        let graph = match context_graph(&context.inner) {
            Ok(graph) => graph,
            Err(error) => {
                set_last_error(error);
                return ptr::null_mut();
            }
        };
        match stream_for_pattern(&model.inner.model, &statement.inner, Some(graph)) {
            Ok(stream) => stream,
            Err(error) => {
                set_last_error(error);
                ptr::null_mut()
            }
        }
    })
}

/// Returns nonzero when the named graph/context has at least one statement.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_contains_context(
    model: *mut librdf_model,
    context: *mut librdf_node,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return 0;
        };
        let Some(context) = (unsafe { borrow_handle(context, crate::handles::TAG_NODE) }) else {
            return 0;
        };
        let graph = match context_graph(&context.inner) {
            Ok(graph) => graph,
            Err(error) => {
                set_last_error(error);
                return 0;
            }
        };
        match model
            .inner
            .model
            .find(StatementPattern {
                graph_name: Some(graph.as_ref()),
                ..StatementPattern::default()
            })
            .next()
        {
            Some(Ok(_)) => 1,
            Some(Err(error)) => {
                set_last_error(error.to_string());
                0
            }
            None => 0,
        }
    })
}

/// Oxiland models support RDF named-graph contexts.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_supports_contexts(model: *mut librdf_model) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        i32::from(unsafe { borrow_handle(model, TAG_MODEL) }.is_some())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_context_add_statements(
    model: *mut librdf_model,
    context: *mut librdf_node,
    stream: *mut librdf_stream,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(stream, TAG_STREAM) }.is_none() {
            return -1;
        }
        while librdf_stream_end(stream) == 0 {
            let stmt = librdf_stream_get_object(stream);
            if librdf_model_context_add_statement(model, context, stmt) != 0 {
                return -1;
            }
            if librdf_stream_next(stream) != 0 {
                break;
            }
        }
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_context_remove_statements(
    model: *mut librdf_model,
    context: *mut librdf_node,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        let Some(context) = (unsafe { borrow_handle(context, TAG_NODE) }) else {
            return -1;
        };
        let graph = match context_graph(&context.inner) {
            Ok(g) => g,
            Err(e) => {
                set_last_error(e);
                return -1;
            }
        };
        model.inner.cardinality.invalidate();
        match model.inner.model.clear_graph(graph) {
            Ok(()) => 0,
            Err(e) => {
                set_last_error(e.to_string());
                -1
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_context_serialize(
    model: *mut librdf_model,
    context: *mut librdf_node,
) -> *mut librdf_stream {
    librdf_model_context_as_stream(model, context)
}
