//! `librdf_model` handle.

use std::ptr;

use oxigraph::model::{NamedNodeRef, NamedOrBlankNodeRef, TermRef, TripleRef};
use oxiland::{Model, OpenOptions, StatementPattern, StorageBackend};

use crate::error::{abort_on_panic, clear_last_error, set_last_error};
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
        let model = match storage.inner.backend {
            StorageBackend::Memory => Model::new(),
            StorageBackend::Fjall => {
                let Some(path) = storage.inner.path.as_ref() else {
                    set_last_error("fjall storage missing path");
                    return ptr::null_mut();
                };
                Model::open_with(OpenOptions::fjall(path))
            }
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
