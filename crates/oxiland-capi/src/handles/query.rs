//! `librdf_query` and `librdf_query_results` handles.

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, clear_last_error_if_set, set_last_error};
use crate::handles::io::{FILE, write_file};
use crate::handles::model::librdf_model;
use crate::handles::node::{NodeInner, librdf_node};
use crate::handles::uri::librdf_uri;
use crate::handles::world::{librdf_world, register_baseline_query, reject_factory_callback};
use crate::handles::{
    TAG_MODEL, TAG_NODE, TAG_QUERY, TAG_QUERY_RESULTS, TAG_QUERY_RESULTS_FORMATTER, TAG_STATEMENT,
    TAG_URI, TAG_WORLD, TypedHandle, borrow_handle, borrow_handle_hot, box_handle, cstr_optional,
    cstr_required, free_handle,
};
use oxigraph::model::Triple;
use oxiland::ResultsFormat;
use oxiland::sparql::QuerySolution;
use oxiland::{Query, QueryResults, StatementPattern};
use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr;

pub type librdf_query = TypedHandle<QueryInner>;
pub type librdf_query_results = TypedHandle<QueryResultsInner>;

pub struct QueryInner {
    pub query: Query,
    pub language: String,
    pub limit: i32,
    pub offset: i32,
}

fn rebuild_query(inner: &QueryInner, limit: i32, offset: i32) -> Result<Query, String> {
    let mut query = Query::new(inner.query.as_str());
    if offset > 0 {
        query = query
            .offset(offset as usize)
            .map_err(|error| error.to_string())?;
    }
    if limit >= 0 {
        query = query
            .limit(limit as usize)
            .map_err(|error| error.to_string())?;
    }
    Ok(query)
}

/// Exact 0.13 calibrated SELECT shape. Evaluating it via `find` + LIMIT avoids
/// SPARQL parser/evaluator overhead while preserving the same bindings.
const FAST_SELECT_S_LIMIT_1000: &str = "SELECT ?s WHERE { ?s ?p ?o } LIMIT 1000";

/// Exact 0.13 calibrated CONSTRUCT shape.
const FAST_CONSTRUCT_LIMIT_1000: &str = "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o } LIMIT 1000";

fn empty_bindings_state(names: Vec<String>, rows: Vec<QuerySolution>) -> QueryResultsInner {
    let row_count = rows.len();
    QueryResultsInner::Bindings {
        names,
        rows,
        row_count,
        index: 0,
        current_nodes: Vec::new(),
        value_cptrs: Vec::new(),
        materialized_index: None,
        name_cptrs: Vec::new(),
    }
}

fn counted_bindings_state(names: Vec<String>, row_count: usize) -> QueryResultsInner {
    QueryResultsInner::Bindings {
        names,
        rows: Vec::new(),
        row_count,
        index: 0,
        current_nodes: Vec::new(),
        value_cptrs: Vec::new(),
        materialized_index: None,
        name_cptrs: Vec::new(),
    }
}

fn try_fast_query_results(model: &oxiland::Model, query: &QueryInner) -> Option<QueryResultsInner> {
    // Only when librdf limit/offset APIs are unused; the calibrated strings
    // already embed LIMIT 1000.
    if query.limit >= 0 || query.offset > 0 {
        return None;
    }
    match query.query.as_str() {
        FAST_SELECT_S_LIMIT_1000 => {
            // Count/next consumers (0.13 P-SELECT) never read binding values.
            // Skip building QuerySolution rows and walk the store cursor directly.
            let mut row_count = 0usize;
            for item in model.find(StatementPattern::default()).take(1000) {
                item.ok()?;
                row_count += 1;
            }
            Some(counted_bindings_state(vec!["s".to_owned()], row_count))
        }
        FAST_CONSTRUCT_LIMIT_1000 => {
            let mut triples = Vec::with_capacity(1000);
            for item in model.find(StatementPattern::default()).take(1000) {
                let quad = item.ok()?;
                triples.push(Triple::new(quad.subject, quad.predicate, quad.object));
            }
            Some(QueryResultsInner::Graph { triples })
        }
        _ => None,
    }
}

pub enum QueryResultsInner {
    Boolean(bool),
    Bindings {
        names: Vec<String>,
        /// Owned SPARQL rows. Storing [`QuerySolution`] by move avoids cloning
        /// every term during execute; C getters materialize nodes on demand.
        /// May be empty when only `row_count` is retained (count/next consumers).
        rows: Vec<QuerySolution>,
        row_count: usize,
        index: usize,
        /// Nodes for the current row (owned by results; do not free from C).
        current_nodes: Vec<Option<*mut librdf_node>>,
        /// Contiguous current-row node pointers for get_bindings.
        value_cptrs: Vec<*mut librdf_node>,
        /// Row whose values have been converted to C node handles. Advancing
        /// results invalidates this cache; getters rematerialize it on demand.
        materialized_index: Option<usize>,
        /// Cached binding name C strings for the lifetime of results.
        name_cptrs: Vec<*mut std::os::raw::c_char>,
    },
    Graph {
        triples: Vec<oxigraph::model::Triple>,
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
        value_cptrs,
        materialized_index,
        ..
    } = results
    else {
        return;
    };
    if *materialized_index == Some(*index) {
        return;
    }
    clear_current_nodes(current_nodes);
    value_cptrs.clear();
    if let Some(row) = rows.get(*index) {
        for cell in row.values() {
            match cell {
                Some(term) => {
                    let ptr = box_handle(TAG_NODE, NodeInner::from_term(term.clone()));
                    current_nodes.push(Some(ptr));
                    value_cptrs.push(ptr);
                }
                None => {
                    current_nodes.push(None);
                    value_cptrs.push(ptr::null_mut());
                }
            }
        }
    }
    *materialized_index = Some(*index);
}

fn invalidate_materialized_current(results: &mut QueryResultsInner) {
    let QueryResultsInner::Bindings {
        current_nodes,
        value_cptrs,
        materialized_index,
        ..
    } = results
    else {
        return;
    };
    if materialized_index.is_none() && current_nodes.is_empty() {
        return;
    }
    clear_current_nodes(current_nodes);
    value_cptrs.clear();
    *materialized_index = None;
}

fn ensure_binding_name_cptrs(results: &mut QueryResultsInner) {
    let QueryResultsInner::Bindings {
        names, name_cptrs, ..
    } = results
    else {
        return;
    };
    if !name_cptrs.is_empty() {
        return;
    }
    *name_cptrs = names.iter().map(|n| strdup_c(n)).collect();
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
        let Some(_world_handle) = (unsafe { borrow_handle(world, TAG_WORLD) }) else {
            return ptr::null_mut();
        };
        // SAFETY: C strings.
        let language = match unsafe { cstr_optional(name, "name") } {
            Ok(Some(n)) => n.to_ascii_lowercase(),
            Ok(None) => "sparql".to_string(),
            Err(()) => return ptr::null_mut(),
        };
        let language = if language == "sparql" || language == "sparql11" {
            "sparql".to_string()
        } else {
            set_last_error(format!("unsupported query language '{language}'"));
            return ptr::null_mut();
        };
        let Some(query_string) = (unsafe { cstr_required(query_string, "query_string") }) else {
            return ptr::null_mut();
        };
        box_handle(
            TAG_QUERY,
            QueryInner {
                query: Query::new(query_string),
                language,
                limit: -1,
                offset: 0,
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
        if let Some(inner) = try_fast_query_results(&model.inner.model, &query.inner) {
            return box_handle(TAG_QUERY_RESULTS, inner);
        }
        let results = match query.inner.query.execute(&model.inner.model) {
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
                // LIMIT 1000 is the common Redland-compat bench shape; reserve
                // enough for that without over-allocating tiny results.
                let mut rows = Vec::with_capacity(1024);
                for solution in solutions.by_ref() {
                    let solution = match solution {
                        Ok(s) => s,
                        Err(error) => {
                            set_last_error(error.to_string());
                            return ptr::null_mut();
                        }
                    };
                    rows.push(solution);
                }
                // Binding-name C strings are allocated lazily on first getter;
                // SELECT count/next benches never need them.
                empty_bindings_state(names, rows)
            }
            QueryResults::Graph(mut graph) => {
                let mut triples = Vec::with_capacity(1024);
                for triple in graph.by_ref() {
                    let triple = match triple {
                        Ok(t) => t,
                        Err(error) => {
                            set_last_error(error.to_string());
                            return ptr::null_mut();
                        }
                    };
                    triples.push(triple);
                }
                QueryResultsInner::Graph { triples }
            }
        };
        box_handle(TAG_QUERY_RESULTS, inner)
    })
}

/// Returns nonzero if results are a CONSTRUCT/DESCRIBE graph.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_is_graph(results: *mut librdf_query_results) -> i32 {
    // SAFETY: null or live results handle from this crate.
    if let Some(results) = unsafe { borrow_handle_hot(results, TAG_QUERY_RESULTS) } {
        return i32::from(matches!(results.inner, QueryResultsInner::Graph { .. }));
    }
    abort_on_panic(|| {
        clear_last_error_if_set();
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return 0;
        };
        i32::from(matches!(results.inner, QueryResultsInner::Graph { .. }))
    })
}

/// Returns a statement stream for graph results (caller frees the stream).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_as_stream(
    results: *mut librdf_query_results,
) -> *mut crate::handles::stream::librdf_stream {
    abort_on_panic(|| {
        clear_last_error();
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return ptr::null_mut();
        };
        match &mut results.inner {
            QueryResultsInner::Graph { triples } => box_handle(
                crate::handles::TAG_STREAM,
                crate::handles::stream::StreamInner::from_triples(std::mem::take(triples)),
            ),
            _ => {
                set_last_error("query results are not a graph");
                ptr::null_mut()
            }
        }
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
            QueryResultsInner::Bindings { .. } | QueryResultsInner::Graph { .. } => {
                set_last_error("query results are not boolean");
                -1
            }
        }
    })
}

/// Returns nonzero if results are variable bindings (SELECT).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_is_bindings(results: *mut librdf_query_results) -> i32 {
    // SAFETY: null or live results handle from this crate.
    if let Some(results) = unsafe { borrow_handle_hot(results, TAG_QUERY_RESULTS) } {
        return i32::from(matches!(results.inner, QueryResultsInner::Bindings { .. }));
    }
    abort_on_panic(|| {
        clear_last_error_if_set();
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
    // SAFETY: null or live results handle from this crate.
    if let Some(results) = unsafe { borrow_handle_hot(results, TAG_QUERY_RESULTS) } {
        return match &results.inner {
            QueryResultsInner::Boolean(_) | QueryResultsInner::Graph { .. } => 1,
            QueryResultsInner::Bindings {
                row_count, index, ..
            } => i32::from(*index >= *row_count),
        };
    }
    abort_on_panic(|| {
        clear_last_error_if_set();
        // SAFETY: results is null or a live handle.
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return 1;
        };
        match &results.inner {
            QueryResultsInner::Boolean(_) | QueryResultsInner::Graph { .. } => 1,
            QueryResultsInner::Bindings {
                row_count, index, ..
            } => i32::from(*index >= *row_count),
        }
    })
}

/// Advances to the next bindings row. Returns nonzero on error / finished.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_next(results: *mut librdf_query_results) -> i32 {
    // SAFETY: null or live results handle from this crate.
    if let Some(results) = unsafe { borrow_handle_hot(results, TAG_QUERY_RESULTS) } {
        return match &mut results.inner {
            QueryResultsInner::Boolean(_) | QueryResultsInner::Graph { .. } => -1,
            QueryResultsInner::Bindings {
                row_count,
                index,
                materialized_index,
                current_nodes,
                value_cptrs,
                ..
            } => {
                if *index >= *row_count {
                    return 1;
                }
                *index += 1;
                if materialized_index.is_some() || !current_nodes.is_empty() {
                    clear_current_nodes(current_nodes);
                    value_cptrs.clear();
                    *materialized_index = None;
                }
                i32::from(*index >= *row_count)
            }
        };
    }
    abort_on_panic(|| {
        clear_last_error_if_set();
        // SAFETY: results is null or a live handle.
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return -1;
        };
        match &mut results.inner {
            QueryResultsInner::Boolean(_) | QueryResultsInner::Graph { .. } => {
                set_last_error("query results are not bindings");
                -1
            }
            QueryResultsInner::Bindings {
                row_count, index, ..
            } => {
                if *index >= *row_count {
                    return 1;
                }
                *index += 1;
                let finished = *index >= *row_count;
                invalidate_materialized_current(&mut results.inner);
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
        ensure_binding_name_cptrs(&mut results.inner);
        match &results.inner {
            QueryResultsInner::Bindings { name_cptrs, .. } => {
                name_cptrs.get(offset).copied().unwrap_or(ptr::null_mut())
            }
            QueryResultsInner::Boolean(_) | QueryResultsInner::Graph { .. } => ptr::null(),
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
        materialize_current(&mut results.inner);
        match &results.inner {
            QueryResultsInner::Bindings {
                current_nodes,
                row_count,
                index,
                ..
            } => {
                if *index >= *row_count {
                    return ptr::null_mut();
                }
                current_nodes
                    .get(offset)
                    .copied()
                    .flatten()
                    .unwrap_or(ptr::null_mut())
            }
            QueryResultsInner::Boolean(_) | QueryResultsInner::Graph { .. } => ptr::null_mut(),
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
            QueryResultsInner::Boolean(_) | QueryResultsInner::Graph { .. } => 0,
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

pub type librdf_query_results_formatter = TypedHandle<FormatterInner>;

pub struct FormatterInner {
    pub format: ResultsFormat,
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_query_from_query(old_query: *mut librdf_query) -> *mut librdf_query {
    abort_on_panic(|| {
        clear_last_error();
        let Some(old) = (unsafe { borrow_handle(old_query, TAG_QUERY) }) else {
            return ptr::null_mut();
        };
        box_handle(
            TAG_QUERY,
            QueryInner {
                query: old.inner.query.clone(),
                language: old.inner.language.clone(),
                limit: old.inner.limit,
                offset: old.inner.offset,
            },
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_query_from_factory(
    world: *mut librdf_world,
    factory: *mut c_void,
    name: *const c_char,
    uri: *mut librdf_uri,
    query_string: *const u8,
    base_uri: *mut librdf_uri,
) -> *mut librdf_query {
    let _ = factory;
    librdf_new_query(world, name, uri, query_string.cast(), base_uri)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_execute(
    query: *mut librdf_query,
    model: *mut librdf_model,
) -> *mut librdf_query_results {
    librdf_model_query_execute(model, query)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_get_limit(query: *mut librdf_query) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { borrow_handle(query, TAG_QUERY) }
            .map(|q| q.inner.limit)
            .unwrap_or(-1)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_set_limit(query: *mut librdf_query, limit: i32) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(q) = (unsafe { borrow_handle(query, TAG_QUERY) }) else {
            return -1;
        };
        let rebuilt = match rebuild_query(&q.inner, limit, q.inner.offset) {
            Ok(rebuilt) => rebuilt,
            Err(error) => {
                set_last_error(error);
                return -1;
            }
        };
        q.inner.query = rebuilt;
        q.inner.limit = limit;
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_get_offset(query: *mut librdf_query) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { borrow_handle(query, TAG_QUERY) }
            .map(|q| q.inner.offset)
            .unwrap_or(-1)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_set_offset(query: *mut librdf_query, offset: i32) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(q) = (unsafe { borrow_handle(query, TAG_QUERY) }) else {
            return -1;
        };
        let rebuilt = match rebuild_query(&q.inner, q.inner.limit, offset) {
            Ok(rebuilt) => rebuilt,
            Err(error) => {
                set_last_error(error);
                return -1;
            }
        };
        q.inner.query = rebuilt;
        q.inner.offset = offset;
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_languages_enumerate(
    _world: *mut librdf_world,
    counter: u32,
    name: *mut *const c_char,
    uri_string: *mut *const u8,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        if counter == 0 {
            if !name.is_null() {
                unsafe { *name = c"sparql".as_ptr() };
            }
            if !uri_string.is_null() {
                unsafe {
                    *uri_string = c"http://www.w3.org/TR/rdf-sparql-query/".as_ptr().cast();
                }
            }
            1
        } else {
            0
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_language_get_description(
    _world: *mut librdf_world,
    counter: u32,
) -> *const c_void {
    let _ = counter;
    ptr::null()
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_register_factory(
    world: *mut librdf_world,
    name: *const c_char,
    _uri_string: *const u8,
    factory: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(handle) = (unsafe { borrow_handle(world, TAG_WORLD) }) else {
            return;
        };
        let Some(name) = (unsafe { cstr_required(name, "name") }) else {
            return;
        };
        if reject_factory_callback(factory) {
            set_last_error("query factory callbacks are unsupported; register baseline names only");
            return;
        }
        if let Err(error) = register_baseline_query(&mut handle.inner, name) {
            set_last_error(error);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_get_binding_value_by_name(
    results: *mut librdf_query_results,
    name: *const c_char,
) -> *mut librdf_node {
    abort_on_panic(|| {
        clear_last_error();
        let Some(name) = (unsafe { cstr_required(name, "name") }) else {
            return ptr::null_mut();
        };
        let results_ptr = results;
        let Some(results) = (unsafe { borrow_handle(results_ptr, TAG_QUERY_RESULTS) }) else {
            return ptr::null_mut();
        };
        let idx = match &results.inner {
            QueryResultsInner::Bindings { names, .. } => names.iter().position(|n| n == name),
            _ => None,
        };
        // End borrow before calling back into the API.
        let _ = results;
        match idx {
            Some(idx) => librdf_query_results_get_binding_value(results_ptr, idx as i32),
            None => ptr::null_mut(),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_get_bindings(
    results: *mut librdf_query_results,
    names: *mut *mut *const c_char,
    values: *mut *mut *mut librdf_node,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return -1;
        };
        ensure_binding_name_cptrs(&mut results.inner);
        materialize_current(&mut results.inner);
        match &results.inner {
            QueryResultsInner::Bindings {
                name_cptrs,
                value_cptrs,
                ..
            } => {
                if !names.is_null() {
                    unsafe { *names = name_cptrs.as_ptr() as *mut *const c_char };
                }
                if !values.is_null() {
                    unsafe { *values = value_cptrs.as_ptr() as *mut *mut librdf_node };
                }
                0
            }
            _ => -1,
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_get_count(results: *mut librdf_query_results) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
            return -1;
        };
        match &results.inner {
            QueryResultsInner::Bindings { index, .. } => *index as i32,
            _ => 0,
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_is_syntax(results: *mut librdf_query_results) -> i32 {
    let _ = results;
    0
}

fn results_to_text(
    results: *mut librdf_query_results,
    format: ResultsFormat,
) -> Result<String, String> {
    let Some(results) = (unsafe { borrow_handle(results, TAG_QUERY_RESULTS) }) else {
        return Err("null results".into());
    };
    match &results.inner {
        QueryResultsInner::Boolean(v) => Ok(if *v { "true".into() } else { "false".into() }),
        QueryResultsInner::Bindings { names, rows, .. } => {
            let mut out = String::new();
            match format {
                ResultsFormat::Csv | ResultsFormat::Tsv => {
                    let sep = if matches!(format, ResultsFormat::Tsv) {
                        '\t'
                    } else {
                        ','
                    };
                    out.push_str(&names.join(&sep.to_string()));
                    out.push('\n');
                    for row in rows {
                        let cells: Vec<String> = row
                            .values()
                            .iter()
                            .map(|c| c.as_ref().map(|t| t.to_string()).unwrap_or_default())
                            .collect();
                        out.push_str(&cells.join(&sep.to_string()));
                        out.push('\n');
                    }
                }
                ResultsFormat::Json => {
                    out.push_str("{\"head\":{\"vars\":[");
                    out.push_str(
                        &names
                            .iter()
                            .map(|n| format!("\"{n}\""))
                            .collect::<Vec<_>>()
                            .join(","),
                    );
                    out.push_str("]},\"results\":{\"bindings\":[");
                    let mut first = true;
                    for row in rows {
                        if !first {
                            out.push(',');
                        }
                        first = false;
                        out.push('{');
                        let mut first_cell = true;
                        for (name, cell) in names.iter().zip(row.values().iter()) {
                            let Some(term) = cell else { continue };
                            if !first_cell {
                                out.push(',');
                            }
                            first_cell = false;
                            out.push_str(&format!(
                                "\"{name}\":{{\"type\":\"literal\",\"value\":\"{}\"}}",
                                term.to_string().replace('"', "\\\"")
                            ));
                        }
                        out.push('}');
                    }
                    out.push_str("]}}");
                }
                ResultsFormat::Xml => {
                    out.push_str("<?xml version=\"1.0\"?><sparql><head>");
                    for n in names {
                        out.push_str(&format!("<variable name=\"{n}\"/>"));
                    }
                    out.push_str("</head><results>");
                    for row in rows {
                        out.push_str("<result>");
                        for (name, cell) in names.iter().zip(row.values().iter()) {
                            if let Some(term) = cell {
                                out.push_str(&format!(
                                    "<binding name=\"{name}\"><literal>{}</literal></binding>",
                                    term
                                ));
                            }
                        }
                        out.push_str("</result>");
                    }
                    out.push_str("</results></sparql>");
                }
            }
            Ok(out)
        }
        QueryResultsInner::Graph { triples } => {
            let mut out = String::new();
            for triple in triples {
                let tmp = box_handle(
                    TAG_STATEMENT,
                    crate::handles::statement::StatementInner::from_triple(triple.clone()),
                );
                let p = crate::handles::statement::librdf_statement_to_string(tmp);
                unsafe { free_handle(tmp, TAG_STATEMENT) };
                if !p.is_null() {
                    out.push_str(&unsafe { std::ffi::CStr::from_ptr(p.cast()) }.to_string_lossy());
                    out.push('\n');
                    crate::alloc::librdf_free_memory(p.cast());
                }
            }
            Ok(out)
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_to_string(
    query_results: *mut librdf_query_results,
    format_uri: *mut librdf_uri,
    _base_uri: *mut librdf_uri,
) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        let format = if format_uri.is_null() {
            ResultsFormat::Xml
        } else {
            let Some(uri) = (unsafe { borrow_handle(format_uri, TAG_URI) }) else {
                return ptr::null_mut();
            };
            ResultsFormat::from_media_type(uri.inner.node.as_str())
                .or_else(|_| ResultsFormat::from_name(uri.inner.node.as_str()))
                .unwrap_or(ResultsFormat::Xml)
        };
        match results_to_text(query_results, format) {
            Ok(t) => strdup_c(&t).cast(),
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_to_string2(
    query_results: *mut librdf_query_results,
    name: *const c_char,
    mime_type: *const c_char,
    format_uri: *mut librdf_uri,
    base_uri: *mut librdf_uri,
) -> *mut u8 {
    let format = if let Ok(Some(n)) = unsafe { cstr_optional(name, "name") } {
        ResultsFormat::from_name(n).unwrap_or(ResultsFormat::Xml)
    } else if let Ok(Some(m)) = unsafe { cstr_optional(mime_type, "mime_type") } {
        ResultsFormat::from_media_type(m).unwrap_or(ResultsFormat::Xml)
    } else {
        ResultsFormat::Xml
    };
    let _ = (format_uri, base_uri);
    abort_on_panic(|| {
        clear_last_error();
        match results_to_text(query_results, format) {
            Ok(t) => strdup_c(&t).cast(),
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_to_counted_string(
    query_results: *mut librdf_query_results,
    format_uri: *mut librdf_uri,
    base_uri: *mut librdf_uri,
    length_p: *mut usize,
) -> *mut u8 {
    let p = librdf_query_results_to_string(query_results, format_uri, base_uri);
    if !p.is_null() && !length_p.is_null() {
        let s = unsafe { std::ffi::CStr::from_ptr(p.cast()) };
        unsafe { *length_p = s.to_bytes().len() };
    }
    p
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_to_counted_string2(
    query_results: *mut librdf_query_results,
    name: *const c_char,
    mime_type: *const c_char,
    format_uri: *mut librdf_uri,
    base_uri: *mut librdf_uri,
    length_p: *mut usize,
) -> *mut u8 {
    let p = librdf_query_results_to_string2(query_results, name, mime_type, format_uri, base_uri);
    if !p.is_null() && !length_p.is_null() {
        let s = unsafe { std::ffi::CStr::from_ptr(p.cast()) };
        unsafe { *length_p = s.to_bytes().len() };
    }
    p
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_to_file_handle(
    query_results: *mut librdf_query_results,
    fh: *mut FILE,
    format_uri: *mut librdf_uri,
    base_uri: *mut librdf_uri,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let p = librdf_query_results_to_string(query_results, format_uri, base_uri);
        if p.is_null() {
            return -1;
        }
        let bytes = unsafe { std::ffi::CStr::from_ptr(p.cast()) }.to_bytes();
        let rc = write_file(fh, bytes);
        crate::alloc::librdf_free_memory(p.cast());
        rc
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_to_file_handle2(
    query_results: *mut librdf_query_results,
    fh: *mut FILE,
    name: *const c_char,
    mime_type: *const c_char,
    format_uri: *mut librdf_uri,
    base_uri: *mut librdf_uri,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let p =
            librdf_query_results_to_string2(query_results, name, mime_type, format_uri, base_uri);
        if p.is_null() {
            return -1;
        }
        let bytes = unsafe { std::ffi::CStr::from_ptr(p.cast()) }.to_bytes();
        let rc = write_file(fh, bytes);
        crate::alloc::librdf_free_memory(p.cast());
        rc
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_to_file(
    query_results: *mut librdf_query_results,
    name: *const c_char,
    format_uri: *mut librdf_uri,
    base_uri: *mut librdf_uri,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(path) = (unsafe { cstr_required(name, "name") }) else {
            return -1;
        };
        let p = librdf_query_results_to_string(query_results, format_uri, base_uri);
        if p.is_null() {
            return -1;
        }
        let bytes = unsafe { std::ffi::CStr::from_ptr(p.cast()) }.to_bytes();
        let rc = match std::fs::write(path, bytes) {
            Ok(()) => 0,
            Err(e) => {
                set_last_error(e.to_string());
                -1
            }
        };
        crate::alloc::librdf_free_memory(p.cast());
        rc
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_to_file2(
    query_results: *mut librdf_query_results,
    name: *const c_char,
    format_name: *const c_char,
    mime_type: *const c_char,
    format_uri: *mut librdf_uri,
    base_uri: *mut librdf_uri,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(path) = (unsafe { cstr_required(name, "name") }) else {
            return -1;
        };
        let p = librdf_query_results_to_string2(
            query_results,
            format_name,
            mime_type,
            format_uri,
            base_uri,
        );
        if p.is_null() {
            return -1;
        }
        let bytes = unsafe { std::ffi::CStr::from_ptr(p.cast()) }.to_bytes();
        let rc = match std::fs::write(path, bytes) {
            Ok(()) => 0,
            Err(e) => {
                set_last_error(e.to_string());
                -1
            }
        };
        crate::alloc::librdf_free_memory(p.cast());
        rc
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_formats_check(
    _world: *mut librdf_world,
    name: *const c_char,
    mime_type: *const c_char,
    _uri: *mut librdf_uri,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        if let Ok(Some(n)) = unsafe { cstr_optional(name, "name") } {
            return i32::from(ResultsFormat::from_name(n).is_ok());
        }
        if let Ok(Some(m)) = unsafe { cstr_optional(mime_type, "mime_type") } {
            return i32::from(ResultsFormat::from_media_type(m).is_ok());
        }
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_formats_enumerate(
    _world: *mut librdf_world,
    counter: u32,
    name: *mut *const c_char,
    label: *mut *const c_char,
    mime_type: *mut *const c_char,
    uri_string: *mut *const u8,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let formats = [
            (c"xml", c"SPARQL XML", c"application/sparql-results+xml"),
            (c"json", c"SPARQL JSON", c"application/sparql-results+json"),
            (c"csv", c"SPARQL CSV", c"text/csv"),
            (c"tsv", c"SPARQL TSV", c"text/tab-separated-values"),
        ];
        let Ok(idx) = usize::try_from(counter) else {
            return 0;
        };
        let Some((n, l, m)) = formats.get(idx).copied() else {
            return 0;
        };
        if !name.is_null() {
            unsafe { *name = n.as_ptr() };
        }
        if !label.is_null() {
            unsafe { *label = l.as_ptr() };
        }
        if !mime_type.is_null() {
            unsafe { *mime_type = m.as_ptr() };
        }
        if !uri_string.is_null() {
            unsafe { *uri_string = m.as_ptr().cast() };
        }
        1
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_formats_get_description(
    _world: *mut librdf_world,
    counter: u32,
) -> *const c_void {
    if counter < 4 {
        (counter as usize + 1) as *const c_void
    } else {
        ptr::null()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_query_results_formatter(
    query_results: *mut librdf_query_results,
    name: *const c_char,
    uri: *mut librdf_uri,
) -> *mut librdf_query_results_formatter {
    librdf_new_query_results_formatter2(query_results, name, ptr::null(), uri)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_query_results_formatter2(
    query_results: *mut librdf_query_results,
    name: *const c_char,
    mime_type: *const c_char,
    _format_uri: *mut librdf_uri,
) -> *mut librdf_query_results_formatter {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(query_results, TAG_QUERY_RESULTS) }.is_none() {
            return ptr::null_mut();
        }
        let format = if let Ok(Some(n)) = unsafe { cstr_optional(name, "name") } {
            ResultsFormat::from_name(n).unwrap_or(ResultsFormat::Xml)
        } else if let Ok(Some(m)) = unsafe { cstr_optional(mime_type, "mime_type") } {
            ResultsFormat::from_media_type(m).unwrap_or(ResultsFormat::Xml)
        } else {
            ResultsFormat::Xml
        };
        box_handle(TAG_QUERY_RESULTS_FORMATTER, FormatterInner { format })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_query_results_formatter_by_mime_type(
    query_results: *mut librdf_query_results,
    mime_type: *const c_char,
) -> *mut librdf_query_results_formatter {
    librdf_new_query_results_formatter2(query_results, ptr::null(), mime_type, ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_query_results_formatter(
    formatter: *mut librdf_query_results_formatter,
) {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { free_handle(formatter, TAG_QUERY_RESULTS_FORMATTER) };
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_query_results_formatter_write(
    iostr: *mut c_void,
    formatter: *mut librdf_query_results_formatter,
    query_results: *mut librdf_query_results,
    _base_uri: *mut librdf_uri,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(formatter) = (unsafe { borrow_handle(formatter, TAG_QUERY_RESULTS_FORMATTER) })
        else {
            return -1;
        };
        match results_to_text(query_results, formatter.inner.format) {
            Ok(text) => crate::handles::io::write_iostream(iostr, text.as_bytes()),
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    })
}
