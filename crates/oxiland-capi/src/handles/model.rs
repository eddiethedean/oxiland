//! `librdf_model` handle.

use std::os::raw::c_char;
use std::ptr;

use oxigraph::model::{GraphName, NamedNodeRef, NamedOrBlankNodeRef, TermRef, TripleRef};
use oxiland::{Model, OpenOptions, StatementPattern, StorageBackend};

use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::node::librdf_node;
use crate::handles::statement::{StatementInner, librdf_statement};
use crate::handles::storage::librdf_storage;
use crate::handles::stream::{StreamInner, librdf_stream};
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_MODEL, TAG_STATEMENT, TAG_STORAGE, TAG_STREAM, TAG_WORLD, TypedHandle, borrow_handle,
    box_handle, free_handle,
};

pub type librdf_model = TypedHandle<ModelInner>;

pub struct ModelInner {
    pub model: Model,
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
                box_handle(TAG_MODEL, ModelInner { model })
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
