//! `librdf_query` and `librdf_query_results` handles.

use std::ptr;

use oxigraph::model::Term;
use oxiland::{Query, QueryResults};

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::model::librdf_model;
use crate::handles::node::{NodeInner, librdf_node};
use crate::handles::uri::librdf_uri;
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_MODEL, TAG_NODE, TAG_QUERY, TAG_QUERY_RESULTS, TAG_WORLD, TypedHandle, borrow_handle,
    box_handle, cstr_optional, cstr_required, free_handle,
};

pub type librdf_query = TypedHandle<QueryInner>;
pub type librdf_query_results = TypedHandle<QueryResultsInner>;

pub struct QueryInner {
    pub text: String,
    pub language: String,
}

pub enum QueryResultsInner {
    Boolean(bool),
    Bindings {
        names: Vec<String>,
        rows: Vec<Vec<Option<Term>>>,
        index: usize,
        /// Nodes for the current row (owned by results; do not free from C).
        current_nodes: Vec<Option<*mut librdf_node>>,
        /// Cached binding name C strings for the lifetime of results.
        name_cptrs: Vec<*mut std::os::raw::c_char>,
    },
}

impl Drop for QueryResultsInner {
    fn drop(&mut self) {
        if let Self::Bindings {
            current_nodes,
            name_cptrs,
            ..
        } = self
        {
            for ptr in current_nodes.drain(..).flatten() {
                // SAFETY: nodes were boxed for this results handle.
                unsafe { free_handle(ptr, TAG_NODE) };
            }
            for ptr in name_cptrs.drain(..) {
                if !ptr.is_null() {
                    // SAFETY: allocated via strdup_c / malloc.
                    unsafe { libc::free(ptr.cast()) };
                }
            }
        }
    }
}

fn clear_current_nodes(nodes: &mut Vec<Option<*mut librdf_node>>) {
    for ptr in nodes.drain(..).flatten() {
        // SAFETY: nodes owned by results.
        unsafe { free_handle(ptr, TAG_NODE) };
    }
}

fn materialize_current(results: &mut QueryResultsInner) {
    let QueryResultsInner::Bindings {
        rows,
        index,
        current_nodes,
        ..
    } = results
    else {
        return;
    };
    clear_current_nodes(current_nodes);
    if let Some(row) = rows.get(*index) {
        for cell in row {
            match cell {
                Some(term) => {
                    let ptr = box_handle(TAG_NODE, NodeInner::from_term(term.clone()));
                    current_nodes.push(Some(ptr));
                }
                None => current_nodes.push(None),
            }
        }
    }
}

/// Creates a SPARQL query (`name` must be `"sparql"` for the preview).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_query(
    world: *mut librdf_world,
    name: *const std::os::raw::c_char,
    _uri: *mut librdf_uri,
    query_string: *const std::os::raw::c_char,
    _query_uri: *mut librdf_uri,
) -> *mut librdf_query {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: world is null or a live world handle.
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        // SAFETY: C strings.
        let language = match unsafe { cstr_optional(name, "name") } {
            Ok(Some(n)) => n.to_ascii_lowercase(),
            Ok(None) => "sparql".to_string(),
            Err(()) => return ptr::null_mut(),
        };
        if language != "sparql" {
            set_last_error(format!("unsupported query language '{language}'"));
            return ptr::null_mut();
        }
        let Some(query_string) = (unsafe { cstr_required(query_string, "query_string") }) else {
            return ptr::null_mut();
        };
        box_handle(
            TAG_QUERY,
            QueryInner {
                text: query_string.to_owned(),
                language,
            },
        )
    })
}

/// Frees a query. Null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_query(query: *mut librdf_query) {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: query is null or a live query handle.
        unsafe { free_handle(query, TAG_QUERY) };
    });
}

/// Executes a query against the model.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_query_execute(
    model: *mut librdf_model,
    query: *mut librdf_query,
) -> *mut librdf_query_results {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: handles are null or live.
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let Some(query) = (unsafe { borrow_handle(query, TAG_QUERY) }) else {
            return ptr::null_mut();
        };
        let results = match Query::new(query.inner.text.clone()).execute(&model.inner.model) {
            Ok(r) => r,
            Err(error) => {
                set_last_error(error.to_string());
                return ptr::null_mut();
            }
        };
        let inner = match results {
            QueryResults::Boolean(value) => QueryResultsInner::Boolean(value),
            QueryResults::Solutions(mut solutions) => {
                let names: Vec<String> = solutions
                    .variables()
                    .iter()
                    .map(|v| v.as_str().to_owned())
                    .collect();
                let mut rows = Vec::new();
                for solution in solutions.by_ref() {
                    let solution = match solution {
                        Ok(s) => s,
                        Err(error) => {
                            set_last_error(error.to_string());
                            return ptr::null_mut();
                        }
                    };
                    let mut row = Vec::with_capacity(names.len());
                    for idx in 0..names.len() {
                        row.push(solution.get(idx).cloned());
                    }
                    rows.push(row);
                }
                let name_cptrs = names.iter().map(|n| strdup_c(n)).collect();
                let mut inner = QueryResultsInner::Bindings {
                    names,
                    rows,
                    index: 0,
                    current_nodes: Vec::new(),
                    name_cptrs,
                };
                materialize_current(&mut inner);
                inner
            }
            QueryResults::Graph(_) => {
                set_last_error(
                    "CONSTRUCT/DESCRIBE results are not exposed on the 0.8 preview query_results API",
                );
                return ptr::null_mut();
            }
        };
        box_handle(TAG_QUERY_RESULTS, inner)
    })
}

/// Returns nonzero if results are boolean (ASK).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_is_boolean(results: *mut librdf_query_results) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: results is null or a live handle.
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return 0;
        };
        i32::from(matches!(results.inner, QueryResultsInner::Boolean(_)))
    })
}

/// Returns the ASK boolean (nonzero if true).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_get_boolean(results: *mut librdf_query_results) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: results is null or a live handle.
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return -1;
        };
        match results.inner {
            QueryResultsInner::Boolean(v) => i32::from(v),
            QueryResultsInner::Bindings { .. } => {
                set_last_error("query results are not boolean");
                -1
            }
        }
    })
}

/// Returns nonzero if results are variable bindings (SELECT).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_is_bindings(results: *mut librdf_query_results) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: results is null or a live handle.
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return 0;
        };
        i32::from(matches!(results.inner, QueryResultsInner::Bindings { .. }))
    })
}

/// Returns nonzero if there is no current bindings row.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_finished(results: *mut librdf_query_results) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: results is null or a live handle.
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return 1;
        };
        match &results.inner {
            QueryResultsInner::Boolean(_) => 1,
            QueryResultsInner::Bindings { rows, index, .. } => i32::from(*index >= rows.len()),
        }
    })
}

/// Advances to the next bindings row. Returns nonzero on error / finished.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_next(results: *mut librdf_query_results) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: results is null or a live handle.
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return -1;
        };
        match &mut results.inner {
            QueryResultsInner::Boolean(_) => {
                set_last_error("query results are not bindings");
                -1
            }
            QueryResultsInner::Bindings { rows, index, .. } => {
                if *index >= rows.len() {
                    return 1;
                }
                *index += 1;
                let finished = *index >= rows.len();
                materialize_current(&mut results.inner);
                i32::from(finished)
            }
        }
    })
}

/// Returns the binding name at `offset` (owned by results).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_get_binding_name(
    results: *mut librdf_query_results,
    offset: i32,
) -> *const std::os::raw::c_char {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: results is null or a live handle.
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return ptr::null();
        };
        let Ok(offset) = usize::try_from(offset) else {
            return ptr::null();
        };
        match &results.inner {
            QueryResultsInner::Bindings { name_cptrs, .. } => {
                name_cptrs.get(offset).copied().unwrap_or(ptr::null_mut())
            }
            QueryResultsInner::Boolean(_) => ptr::null(),
        }
    })
}

/// Returns the binding value at `offset` (owned by results; do not free).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_get_binding_value(
    results: *mut librdf_query_results,
    offset: i32,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: results is null or a live handle.
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return ptr::null_mut();
        };
        let Ok(offset) = usize::try_from(offset) else {
            return ptr::null_mut();
        };
        match &results.inner {
            QueryResultsInner::Bindings {
                current_nodes,
                rows,
                index,
                ..
            } => {
                if *index >= rows.len() {
                    return ptr::null_mut();
                }
                current_nodes
                    .get(offset)
                    .copied()
                    .flatten()
                    .unwrap_or(ptr::null_mut())
            }
            QueryResultsInner::Boolean(_) => ptr::null_mut(),
        }
    })
}

/// Returns the number of bindings (variables), or negative on error.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_get_bindings_count(
    results: *mut librdf_query_results,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: results is null or a live handle.
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return -1;
        };
        match &results.inner {
            QueryResultsInner::Bindings { names, .. } => {
                i32::try_from(names.len()).unwrap_or(i32::MAX)
            }
            QueryResultsInner::Boolean(_) => 0,
        }
    })
}

/// Frees query results. Null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_query_results(results: *mut librdf_query_results) {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: results is null or a live handle.
        unsafe { free_handle(results, TAG_QUERY_RESULTS) };
    });
}
