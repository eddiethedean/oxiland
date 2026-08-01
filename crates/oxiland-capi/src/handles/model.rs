//! `librdf_model` handle.

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::hash::librdf_hash;
use crate::handles::io::{FILE, write_iostream, writeln_file};
use crate::handles::iterator::{box_items, librdf_iterator};
use crate::handles::node::NodeInner;
use crate::handles::node::librdf_node;
use crate::handles::statement::{StatementInner, librdf_statement};
use crate::handles::storage::librdf_storage;
use crate::handles::stream::{StreamInner, librdf_stream};
use crate::handles::stream::{librdf_stream_end, librdf_stream_get_object, librdf_stream_next};
use crate::handles::uri::librdf_uri;
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_MODEL, TAG_STATEMENT, TAG_STORAGE, TAG_STREAM, TAG_WORLD, TypedHandle, borrow_handle,
    box_handle, free_handle,
};
use crate::handles::{TAG_NODE, TAG_URI, cstr_optional};
use oxigraph::model::Term;
use oxigraph::model::{GraphName, NamedNodeRef, NamedOrBlankNodeRef, TermRef, TripleRef};
use oxiland::io::{Parser, Serializer, Syntax};
use oxiland::{Model, OpenOptions, StatementPattern, StorageBackend};
use std::collections::HashSet;
use std::ffi::c_void;
use std::io::Cursor;
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;

pub type librdf_model = TypedHandle<ModelInner>;

pub struct ModelInner {
    pub model: Model,
    pub storage: *mut librdf_storage,
    pub features: std::collections::HashMap<String, String>,
    pub in_transaction: bool,
    pub transaction_handle: *mut std::ffi::c_void,
}

/// Creates a model from storage (`memory` → [`Model::new`], `fjall` → open path).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_model(
    world: *mut librdf_world,
    storage: *mut librdf_storage,
    _options: *const std::os::raw::c_char,
) -> *mut librdf_model {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: world/storage are null or live handles.
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        let Some(storage) = (unsafe { borrow_handle(storage, TAG_STORAGE) }) else {
            return ptr::null_mut();
        };
        let model = if storage.inner.backend == StorageBackend::Memory {
            Model::new()
        } else {
            let Some(path) = storage.inner.path.as_ref() else {
                set_last_error(format!(
                    "{} storage missing path",
                    storage.inner.backend.name()
                ));
                return ptr::null_mut();
            };
            Model::open_with(OpenOptions::new(storage.inner.backend, path))
        };
        match model {
            Ok(model) => {
                storage.inner.opened = true;
                let handle = box_handle(
                    TAG_MODEL,
                    ModelInner {
                        model,
                        storage,
                        features: std::collections::HashMap::new(),
                        in_transaction: false,
                        transaction_handle: ptr::null_mut(),
                    },
                );
                storage.inner.model = Some(handle);
                handle
            }
            Err(error) => {
                set_last_error(error.to_string());
                ptr::null_mut()
            }
        }
    })
}

/// Frees a model. Null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_model(model: *mut librdf_model) {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: model is null or a live model handle.
        unsafe { free_handle(model, TAG_MODEL) };
    });
}

fn statement_as_triple(stmt: &StatementInner) -> Result<oxigraph::model::Triple, String> {
    let subject = stmt
        .subject
        .as_ref()
        .ok_or_else(|| "statement subject is null".to_string())?
        .as_named_or_blank()
        .ok_or_else(|| "statement subject must be IRI or blank".to_string())?;
    let predicate = stmt
        .predicate
        .as_ref()
        .ok_or_else(|| "statement predicate is null".to_string())?
        .as_named()
        .ok_or_else(|| "statement predicate must be IRI".to_string())?;
    let object = stmt
        .object
        .as_ref()
        .ok_or_else(|| "statement object is null".to_string())?
        .term
        .clone();
    Ok(oxigraph::model::Triple::new(subject, predicate, object))
}

fn context_graph(context: &crate::handles::node::NodeInner) -> Result<GraphName, String> {
    context
        .as_named()
        .map(GraphName::NamedNode)
        .ok_or_else(|| "context must be an IRI".to_string())
}

fn stream_for_pattern(
    model: &Model,
    statement: &StatementInner,
    graph_name: Option<GraphName>,
) -> Result<*mut librdf_stream, String> {
    let subject_owned = statement
        .subject
        .as_ref()
        .and_then(|node| node.as_named_or_blank());
    let predicate_owned = statement
        .predicate
        .as_ref()
        .and_then(|node| node.as_named());
    let object_owned = statement.object.as_ref().map(|node| node.term.clone());
    let pattern = StatementPattern {
        subject: subject_owned.as_ref().map(NamedOrBlankNodeRef::from),
        predicate: predicate_owned.as_ref().map(NamedNodeRef::from),
        object: object_owned.as_ref().map(TermRef::from),
        graph_name: graph_name.as_ref().map(|name| name.as_ref()),
    };
    let mut statements = Vec::new();
    for item in model.find(pattern) {
        let quad = item.map_err(|error| error.to_string())?;
        let triple = oxigraph::model::Triple::new(quad.subject, quad.predicate, quad.object);
        statements.push(StatementInner::from_triple(triple));
    }
    Ok(box_handle(
        TAG_STREAM,
        StreamInner {
            statements,
            index: 0,
            current: None,
        },
    ))
}

/// Adds a statement. Returns nonzero on error.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_add_statement(
    model: *mut librdf_model,
    statement: *mut librdf_statement,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: model/statement are null or live handles.
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return -1;
        };
        let triple = match statement_as_triple(&statement.inner) {
            Ok(t) => t,
            Err(msg) => {
                set_last_error(msg);
                return -1;
            }
        };
        match model.inner.model.add(triple) {
            Ok(_) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

/// Removes a statement. Returns nonzero on error.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_remove_statement(
    model: *mut librdf_model,
    statement: *mut librdf_statement,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: model/statement are null or live handles.
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return -1;
        };
        let triple = match statement_as_triple(&statement.inner) {
            Ok(t) => t,
            Err(msg) => {
                set_last_error(msg);
                return -1;
            }
        };
        match model.inner.model.remove(triple) {
            Ok(_) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

/// Returns nonzero if the model contains the statement.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_contains_statement(
    model: *mut librdf_model,
    statement: *mut librdf_statement,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: model/statement are null or live handles.
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return 0;
        };
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return 0;
        };
        let triple = match statement_as_triple(&statement.inner) {
            Ok(t) => t,
            Err(msg) => {
                set_last_error(msg);
                return 0;
            }
        };
        match model.inner.model.contains(TripleRef::from(&triple)) {
            Ok(true) => 1,
            Ok(false) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                0
            }
        }
    })
}

/// Returns the number of statements, or negative on error.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_size(model: *mut librdf_model) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: model is null or a live model handle.
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        match model.inner.model.len() {
            Ok(n) => i32::try_from(n).unwrap_or(i32::MAX),
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

/// Finds statements matching `statement` (NULL node fields are wildcards).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_find_statements(
    model: *mut librdf_model,
    statement: *mut librdf_statement,
) -> *mut librdf_stream {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: model/statement are null or live handles.
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let Some(statement) = (unsafe { borrow_handle(statement, TAG_STATEMENT) }) else {
            return ptr::null_mut();
        };

        let subject_owned = statement
            .inner
            .subject
            .as_ref()
            .and_then(|n| n.as_named_or_blank());
        let predicate_owned = statement
            .inner
            .predicate
            .as_ref()
            .and_then(|n| n.as_named());
        let object_owned = statement.inner.object.as_ref().map(|n| n.term.clone());

        let pattern = StatementPattern {
            subject: subject_owned.as_ref().map(NamedOrBlankNodeRef::from),
            predicate: predicate_owned.as_ref().map(NamedNodeRef::from),
            object: object_owned.as_ref().map(TermRef::from),
            graph_name: None,
        };

        let mut statements = Vec::new();
        for item in model.inner.model.find(pattern) {
            match item {
                Ok(quad) => {
                    let triple =
                        oxigraph::model::Triple::new(quad.subject, quad.predicate, quad.object);
                    statements.push(StatementInner::from_triple(triple));
                }
                Err(error) => {
                    set_last_error(error.to_string());
                    return ptr::null_mut();
                }
            }
        }

        box_handle(
            TAG_STREAM,
            StreamInner {
                statements,
                index: 0,
                current: None,
            },
        )
    })
}

/// Syncs durable storage. Returns nonzero on error.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_sync(model: *mut librdf_model) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        match model.inner.model.sync() {
            Ok(()) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

/// Returns a stream of all statements in the model.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_as_stream(model: *mut librdf_model) -> *mut librdf_stream {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let mut statements = Vec::new();
        for item in model.inner.model.find(StatementPattern::default()) {
            match item {
                Ok(quad) => {
                    let triple =
                        oxigraph::model::Triple::new(quad.subject, quad.predicate, quad.object);
                    statements.push(StatementInner::from_triple(triple));
                }
                Err(error) => {
                    set_last_error(error.to_string());
                    return ptr::null_mut();
                }
            }
        }
        box_handle(
            TAG_STREAM,
            StreamInner {
                statements,
                index: 0,
                current: None,
            },
        )
    })
}

/// Adds a triple from three nodes (does not take ownership of the nodes).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_add(
    model: *mut librdf_model,
    subject: *mut crate::handles::node::librdf_node,
    predicate: *mut crate::handles::node::librdf_node,
    object: *mut crate::handles::node::librdf_node,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        let Some(subject) = (unsafe { borrow_handle(subject, crate::handles::TAG_NODE) }) else {
            return -1;
        };
        let Some(predicate) = (unsafe { borrow_handle(predicate, crate::handles::TAG_NODE) })
        else {
            return -1;
        };
        let Some(object) = (unsafe { borrow_handle(object, crate::handles::TAG_NODE) }) else {
            return -1;
        };
        let Some(s) = subject.inner.as_named_or_blank() else {
            set_last_error("subject must be IRI or blank");
            return -1;
        };
        let Some(p) = predicate.inner.as_named() else {
            set_last_error("predicate must be IRI");
            return -1;
        };
        let triple = oxigraph::model::Triple::new(s, p, object.inner.term.clone());
        match model.inner.model.add(triple) {
            Ok(_) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

/// Adds a plain or language-tagged literal statement.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_add_string_literal_statement(
    model: *mut librdf_model,
    subject: *mut librdf_node,
    predicate: *mut librdf_node,
    literal: *const u8,
    xml_language: *const c_char,
    _is_wf_xml: i32,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(literal) = (unsafe { crate::handles::cstr_required(literal.cast(), "literal") })
        else {
            return -1;
        };
        let language = match unsafe { crate::handles::cstr_optional(xml_language, "xml_language") }
        {
            Ok(language) => language,
            Err(()) => return -1,
        };
        let object = match language.filter(|language| !language.is_empty()) {
            Some(language) => {
                match oxigraph::model::Literal::new_language_tagged_literal(literal, language) {
                    Ok(value) => value,
                    Err(error) => {
                        set_last_error(error.to_string());
                        return -1;
                    }
                }
            }
            None => oxigraph::model::Literal::new_simple_literal(literal),
        };
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        let Some(subject) = (unsafe { borrow_handle(subject, crate::handles::TAG_NODE) }) else {
            return -1;
        };
        let Some(predicate) = (unsafe { borrow_handle(predicate, crate::handles::TAG_NODE) })
        else {
            return -1;
        };
        let Some(subject) = subject.inner.as_named_or_blank() else {
            set_last_error("subject must be IRI or blank");
            return -1;
        };
        let Some(predicate) = predicate.inner.as_named() else {
            set_last_error("predicate must be IRI");
            return -1;
        };
        match model
            .inner
            .model
            .add(oxigraph::model::Triple::new(subject, predicate, object))
        {
            Ok(_) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

/// Adds a typed literal statement. Language-tagged typed literals are rejected by RDF 1.1.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_add_typed_literal_statement(
    model: *mut librdf_model,
    subject: *mut librdf_node,
    predicate: *mut librdf_node,
    literal: *const u8,
    xml_language: *const c_char,
    datatype_uri: *mut crate::handles::uri::librdf_uri,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(literal) = (unsafe { crate::handles::cstr_required(literal.cast(), "literal") })
        else {
            return -1;
        };
        let language = match unsafe { crate::handles::cstr_optional(xml_language, "xml_language") }
        {
            Ok(language) => language,
            Err(()) => return -1,
        };
        if language.is_some_and(|language| !language.is_empty()) {
            set_last_error("a typed literal cannot have a language tag");
            return -1;
        }
        let Some(datatype) = (unsafe { borrow_handle(datatype_uri, crate::handles::TAG_URI) })
        else {
            return -1;
        };
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        let Some(subject) = (unsafe { borrow_handle(subject, crate::handles::TAG_NODE) }) else {
            return -1;
        };
        let Some(predicate) = (unsafe { borrow_handle(predicate, crate::handles::TAG_NODE) })
        else {
            return -1;
        };
        let Some(subject) = subject.inner.as_named_or_blank() else {
            set_last_error("subject must be IRI or blank");
            return -1;
        };
        let Some(predicate) = predicate.inner.as_named() else {
            set_last_error("predicate must be IRI");
            return -1;
        };
        let object =
            oxigraph::model::Literal::new_typed_literal(literal, datatype.inner.node.clone());
        match model
            .inner
            .model
            .add(oxigraph::model::Triple::new(subject, predicate, object))
        {
            Ok(_) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

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

/// Serializes the model as Turtle to a malloc'd string.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_to_string(
    model: *mut librdf_model,
    _base_uri: *mut crate::handles::uri::librdf_uri,
) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        match oxiland::io::Serializer::for_syntax(oxiland::io::Syntax::Turtle)
            .serialize_model_to_string(&model.inner.model)
        {
            Ok(text) => crate::alloc::strdup_c(&text).cast(),
            Err(error) => {
                set_last_error(error.to_string());
                ptr::null_mut()
            }
        }
    })
}

/// Executes a SPARQL Update string against the model.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_update(model: *mut librdf_model, update_string: *const u8) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        let Some(update_string) =
            (unsafe { crate::handles::cstr_required(update_string.cast(), "update_string") })
        else {
            return -1;
        };
        match oxiland::Update::new(update_string).execute(&model.inner.model) {
            Ok(()) => 0,
            Err(error) => {
                set_last_error(error.to_string());
                -1
            }
        }
    })
}

fn node_term(node: *mut librdf_node) -> Option<Term> {
    let n = unsafe { borrow_handle(node, TAG_NODE) }?;
    Some(n.inner.term.clone())
}

fn collect_matching_nodes(
    model: &Model,
    want_subject: Option<&Term>,
    want_predicate: Option<&Term>,
    want_object: Option<&Term>,
    project: fn(&oxigraph::model::Quad) -> Term,
) -> Result<Vec<*mut c_void>, String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for item in model.find(StatementPattern::default()) {
        let quad = item.map_err(|e| e.to_string())?;
        let s = Term::from(quad.subject.clone());
        let p = Term::from(quad.predicate.clone());
        let o = quad.object.clone();
        if want_subject.is_some_and(|t| t != &s) {
            continue;
        }
        if want_predicate.is_some_and(|t| t != &p) {
            continue;
        }
        if want_object.is_some_and(|t| t != &o) {
            continue;
        }
        let term = project(&quad);
        if seen.insert(term.to_string()) {
            let ptr = box_handle(TAG_NODE, NodeInner::from_term(term));
            out.push(ptr.cast());
        }
    }
    Ok(out)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_model_with_options(
    world: *mut librdf_world,
    storage: *mut librdf_storage,
    _options: *mut librdf_hash,
) -> *mut librdf_model {
    librdf_new_model(world, storage, ptr::null())
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_model_from_model(model: *mut librdf_model) -> *mut librdf_model {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        // Memory snapshot via N-Quads round-trip through a fresh memory model.
        let world = unsafe { borrow_handle(model.inner.storage, TAG_STORAGE) }
            .map(|s| s.inner.world)
            .unwrap_or(ptr::null_mut());
        let storage = crate::handles::storage::librdf_new_storage(
            world,
            c"memory".as_ptr(),
            ptr::null(),
            ptr::null(),
        );
        if storage.is_null() {
            return ptr::null_mut();
        }
        let new_model = librdf_new_model(world, storage, ptr::null());
        if new_model.is_null() {
            crate::handles::storage::librdf_free_storage(storage);
            return ptr::null_mut();
        }
        let stream = librdf_model_as_stream(model as *mut _);
        // Re-borrow after creating new model
        let _ = librdf_model_add_statements(new_model, stream);
        crate::handles::stream::librdf_free_stream(stream);
        new_model
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_enumerate(
    _world: *mut librdf_world,
    counter: u32,
    name: *mut *const c_char,
    label: *mut *const c_char,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        if counter == 0 {
            if !name.is_null() {
                unsafe { *name = c"memory".as_ptr() };
            }
            if !label.is_null() {
                unsafe { *label = c"memory".as_ptr() };
            }
            1
        } else {
            0
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_storage(model: *mut librdf_model) -> *mut librdf_storage {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { borrow_handle(model, TAG_MODEL) }
            .map(|m| m.inner.storage)
            .unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_add_statements(
    model: *mut librdf_model,
    statement_stream: *mut librdf_stream,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(model, TAG_MODEL) }.is_none() {
            return -1;
        }
        if unsafe { borrow_handle(statement_stream, TAG_STREAM) }.is_none() {
            return -1;
        }
        while librdf_stream_end(statement_stream) == 0 {
            let stmt = librdf_stream_get_object(statement_stream);
            if librdf_model_add_statement(model, stmt) != 0 {
                return -1;
            }
            if librdf_stream_next(statement_stream) != 0 {
                break;
            }
        }
        0
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

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_find_statements_with_options(
    model: *mut librdf_model,
    statement: *mut librdf_statement,
    context_node: *mut librdf_node,
    _options: *mut librdf_hash,
) -> *mut librdf_stream {
    if context_node.is_null() {
        librdf_model_find_statements(model, statement)
    } else {
        librdf_model_find_statements_in_context(model, statement, context_node)
    }
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
        let pred = node_term(arc);
        let obj = node_term(target);
        match collect_matching_nodes(&model.inner.model, None, pred.as_ref(), obj.as_ref(), |q| {
            Term::from(q.subject.clone())
        }) {
            Ok(items) => box_items(items),
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
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
        let subj = node_term(source);
        let obj = node_term(target);
        match collect_matching_nodes(&model.inner.model, subj.as_ref(), None, obj.as_ref(), |q| {
            Term::from(q.predicate.clone())
        }) {
            Ok(items) => box_items(items),
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
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
        let subj = node_term(source);
        let pred = node_term(arc);
        match collect_matching_nodes(
            &model.inner.model,
            subj.as_ref(),
            pred.as_ref(),
            None,
            |q| q.object.clone(),
        ) {
            Ok(items) => box_items(items),
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_source(
    model: *mut librdf_model,
    arc: *mut librdf_node,
    target: *mut librdf_node,
) -> *mut librdf_node {
    let it = librdf_model_get_sources(model, arc, target);
    if it.is_null() {
        return ptr::null_mut();
    }
    let obj = crate::handles::iterator::librdf_iterator_get_object(it);
    // Caller owns returned node; leave iterator's item (shared ownership issue).
    // Clone via term:
    let node = if obj.is_null() {
        ptr::null_mut()
    } else {
        let term = unsafe { borrow_handle(obj.cast::<librdf_node>(), TAG_NODE) }
            .map(|n| n.inner.term.clone());
        match term {
            Some(t) => box_handle(TAG_NODE, NodeInner::from_term(t)),
            None => ptr::null_mut(),
        }
    };
    crate::handles::iterator::librdf_free_iterator(it);
    node
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_arc(
    model: *mut librdf_model,
    source: *mut librdf_node,
    target: *mut librdf_node,
) -> *mut librdf_node {
    let it = librdf_model_get_arcs(model, source, target);
    if it.is_null() {
        return ptr::null_mut();
    }
    let obj = crate::handles::iterator::librdf_iterator_get_object(it);
    let node = if obj.is_null() {
        ptr::null_mut()
    } else {
        let term = unsafe { borrow_handle(obj.cast::<librdf_node>(), TAG_NODE) }
            .map(|n| n.inner.term.clone());
        match term {
            Some(t) => box_handle(TAG_NODE, NodeInner::from_term(t)),
            None => ptr::null_mut(),
        }
    };
    crate::handles::iterator::librdf_free_iterator(it);
    node
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_get_target(
    model: *mut librdf_model,
    source: *mut librdf_node,
    arc: *mut librdf_node,
) -> *mut librdf_node {
    let it = librdf_model_get_targets(model, source, arc);
    if it.is_null() {
        return ptr::null_mut();
    }
    let obj = crate::handles::iterator::librdf_iterator_get_object(it);
    let node = if obj.is_null() {
        ptr::null_mut()
    } else {
        let term = unsafe { borrow_handle(obj.cast::<librdf_node>(), TAG_NODE) }
            .map(|n| n.inner.term.clone());
        match term {
            Some(t) => box_handle(TAG_NODE, NodeInner::from_term(t)),
            None => ptr::null_mut(),
        }
    };
    crate::handles::iterator::librdf_free_iterator(it);
    node
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
        let obj = node_term(node);
        match collect_matching_nodes(&model.inner.model, None, None, obj.as_ref(), |q| {
            Term::from(q.predicate.clone())
        }) {
            Ok(items) => box_items(items),
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
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
        let subj = node_term(node);
        match collect_matching_nodes(&model.inner.model, subj.as_ref(), None, None, |q| {
            Term::from(q.predicate.clone())
        }) {
            Ok(items) => box_items(items),
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_has_arc_in(
    model: *mut librdf_model,
    node: *mut librdf_node,
    property: *mut librdf_node,
) -> i32 {
    let it = librdf_model_get_arcs_in(model, node);
    if it.is_null() {
        return 0;
    }
    let mut found = 0;
    let want = node_term(property);
    while crate::handles::iterator::librdf_iterator_end(it) == 0 {
        let obj = crate::handles::iterator::librdf_iterator_get_object(it);
        if let (Some(want), Some(got)) = (
            want.as_ref(),
            unsafe { borrow_handle(obj.cast::<librdf_node>(), TAG_NODE) }
                .map(|n| n.inner.term.clone()),
        ) {
            if want == &got {
                found = 1;
                break;
            }
        }
        if crate::handles::iterator::librdf_iterator_next(it) != 0 {
            break;
        }
    }
    crate::handles::iterator::librdf_free_iterator(it);
    found
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_has_arc_out(
    model: *mut librdf_model,
    node: *mut librdf_node,
    property: *mut librdf_node,
) -> i32 {
    let it = librdf_model_get_arcs_out(model, node);
    if it.is_null() {
        return 0;
    }
    let mut found = 0;
    let want = node_term(property);
    while crate::handles::iterator::librdf_iterator_end(it) == 0 {
        let obj = crate::handles::iterator::librdf_iterator_get_object(it);
        if let (Some(want), Some(got)) = (
            want.as_ref(),
            unsafe { borrow_handle(obj.cast::<librdf_node>(), TAG_NODE) }
                .map(|n| n.inner.term.clone()),
        ) {
            if want == &got {
                found = 1;
                break;
            }
        }
        if crate::handles::iterator::librdf_iterator_next(it) != 0 {
            break;
        }
    }
    crate::handles::iterator::librdf_free_iterator(it);
    found
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
                    if let GraphName::NamedNode(n) = quad.graph_name {
                        if seen.insert(n.as_str().to_owned()) {
                            items.push(
                                box_handle(TAG_NODE, NodeInner::from_term(Term::NamedNode(n)))
                                    .cast(),
                            );
                        }
                    }
                }
                Err(e) => {
                    set_last_error(e.to_string());
                    return ptr::null_mut();
                }
            }
        }
        box_items(items)
    })
}

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

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_load(
    model: *mut librdf_model,
    uri: *mut librdf_uri,
    name: *const c_char,
    mime_type: *const c_char,
    _type_uri: *mut librdf_uri,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        let Some(uri) = (unsafe { borrow_handle(uri, TAG_URI) }) else {
            return -1;
        };
        let syntax_name = unsafe { cstr_optional(name, "name") }.ok().flatten();
        let mime = unsafe { cstr_optional(mime_type, "mime_type") }
            .ok()
            .flatten();
        let syntax = if let Some(n) = syntax_name {
            Syntax::from_name(n).unwrap_or(Syntax::Turtle)
        } else if let Some(m) = mime {
            Syntax::from_media_type(m).unwrap_or(Syntax::Turtle)
        } else {
            Syntax::Turtle
        };
        let path = match oxiland::utility::file_uri_to_path(uri.inner.node.as_str()) {
            Ok(p) => p,
            Err(_) => {
                // Treat as local path string
                Path::new(uri.inner.node.as_str()).to_path_buf()
            }
        };
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                set_last_error(e.to_string());
                return -1;
            }
        };
        match Parser::for_syntax(syntax).load_into(&model.inner.model, Cursor::new(bytes)) {
            Ok(_) => 0,
            Err(e) => {
                set_last_error(e.to_string());
                -1
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_to_counted_string(
    model: *mut librdf_model,
    _uri: *mut librdf_uri,
    name: *const c_char,
    mime_type: *const c_char,
    _type_uri: *mut librdf_uri,
    string_length_p: *mut usize,
) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return ptr::null_mut();
        };
        let syntax_name = unsafe { cstr_optional(name, "name") }.ok().flatten();
        let mime = unsafe { cstr_optional(mime_type, "mime_type") }
            .ok()
            .flatten();
        let syntax = if let Some(n) = syntax_name {
            Syntax::from_name(n).unwrap_or(Syntax::Turtle)
        } else if let Some(m) = mime {
            Syntax::from_media_type(m).unwrap_or(Syntax::Turtle)
        } else {
            Syntax::Turtle
        };
        match Serializer::for_syntax(syntax).serialize_model_to_string(&model.inner.model) {
            Ok(text) => {
                if !string_length_p.is_null() {
                    unsafe { *string_length_p = text.len() };
                }
                strdup_c(&text).cast()
            }
            Err(e) => {
                set_last_error(e.to_string());
                ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_print(model: *mut librdf_model, fh: *mut FILE) {
    abort_on_panic(|| {
        clear_last_error();
        let text_ptr = librdf_model_to_string(model, ptr::null_mut());
        if text_ptr.is_null() {
            return;
        }
        let text = unsafe { std::ffi::CStr::from_ptr(text_ptr.cast()) }.to_string_lossy();
        let _ = writeln_file(fh, &text);
        crate::alloc::librdf_free_memory(text_ptr.cast());
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_write(model: *mut librdf_model, iostr: *mut c_void) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let text_ptr = librdf_model_to_string(model, ptr::null_mut());
        if text_ptr.is_null() {
            return -1;
        }
        let text = unsafe { std::ffi::CStr::from_ptr(text_ptr.cast()) }.to_bytes();
        let rc = write_iostream(iostr, text);
        crate::alloc::librdf_free_memory(text_ptr.cast());
        rc
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_serialise(model: *mut librdf_model) -> *mut librdf_stream {
    librdf_model_as_stream(model)
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_add_submodel(
    model: *mut librdf_model,
    sub_model: *mut librdf_model,
) -> i32 {
    let stream = librdf_model_as_stream(sub_model);
    if stream.is_null() {
        return -1;
    }
    let rc = librdf_model_add_statements(model, stream);
    crate::handles::stream::librdf_free_stream(stream);
    rc
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_remove_submodel(
    model: *mut librdf_model,
    sub_model: *mut librdf_model,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let stream = librdf_model_as_stream(sub_model);
        if stream.is_null() {
            return -1;
        }
        while librdf_stream_end(stream) == 0 {
            let stmt = librdf_stream_get_object(stream);
            let _ = librdf_model_remove_statement(model, stmt);
            if librdf_stream_next(stream) != 0 {
                break;
            }
        }
        crate::handles::stream::librdf_free_stream(stream);
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_transaction_start(model: *mut librdf_model) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        if model.inner.in_transaction {
            set_last_error("transaction already active");
            return -1;
        }
        model.inner.in_transaction = true;
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_transaction_start_with_handle(
    model: *mut librdf_model,
    handle: *mut c_void,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        model.inner.in_transaction = true;
        model.inner.transaction_handle = handle;
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_transaction_commit(model: *mut librdf_model) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        model.inner.in_transaction = false;
        model.inner.transaction_handle = ptr::null_mut();
        match model.inner.model.sync() {
            Ok(()) => 0,
            Err(e) => {
                set_last_error(e.to_string());
                -1
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_transaction_rollback(model: *mut librdf_model) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(model) = (unsafe { borrow_handle(model, TAG_MODEL) }) else {
            return -1;
        };
        model.inner.in_transaction = false;
        model.inner.transaction_handle = ptr::null_mut();
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_model_transaction_get_handle(model: *mut librdf_model) -> *mut c_void {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { borrow_handle(model, TAG_MODEL) }
            .map(|m| m.inner.transaction_handle)
            .unwrap_or(ptr::null_mut())
    })
}
