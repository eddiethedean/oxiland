//! Shared backend conformance suite (ADR-022 / SB-03).
//!
//! Registers memory and Fjall through one harness so optional adapters can join
//! the same cases in 0.9 without duplicating Model-level expectations.

use oxigraph::model::{GraphName, Quad};
use oxiland::terms::{self, Literal, Triple};
use oxiland::{Error, Model, OpenOptions, StatementPattern, StorageBackend, compiled_backends};

fn statement(object: &str) -> Triple {
    Triple::new(
        terms::named_node("https://example.com/s").unwrap(),
        terms::named_node("https://example.com/p").unwrap(),
        Literal::new_simple_literal(object),
    )
}

fn open_registered(backend: StorageBackend, path: &std::path::Path) -> Model {
    match backend {
        StorageBackend::Memory => Model::open_with(OpenOptions::new(backend, path)).unwrap(),
        StorageBackend::Fjall => {
            Model::open_with(OpenOptions::new(backend, path).create(true)).unwrap()
        }
    }
}

#[test]
fn compiled_backends_are_memory_and_fjall() {
    assert_eq!(
        compiled_backends(),
        &[StorageBackend::Memory, StorageBackend::Fjall]
    );
}

#[test]
fn registry_rejects_known_uncompiled_and_unknown() {
    let err = StorageBackend::from_name("redb").unwrap_err();
    assert!(matches!(err, Error::Unsupported(msg) if msg.contains("not compiled")));
    let err = StorageBackend::from_name("not-a-backend").unwrap_err();
    assert!(matches!(err, Error::Unsupported(msg) if msg.contains("not recognized")));
}

#[test]
fn harness_crud_commit_and_clear_for_each_compiled_backend() {
    for backend in compiled_backends() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(backend.name());
        let model = open_registered(*backend, &path);
        assert_eq!(model.backend(), *backend);
        assert!(model.add(statement("a")).unwrap());
        assert!(model.contains(statement("a").as_ref()).unwrap());
        assert_eq!(model.len().unwrap(), 1);
        model
            .transaction(|tx| {
                tx.add(statement("b"))?;
                Ok(())
            })
            .unwrap();
        assert_eq!(model.len().unwrap(), 2);
        model.clear().unwrap();
        assert_eq!(model.len().unwrap(), 0);
    }
}

#[test]
fn harness_find_and_bulk_insert_for_each_compiled_backend() {
    for backend in compiled_backends() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(backend.name());
        let model = open_registered(*backend, &path);
        let quads: Vec<_> = ["x", "y", "z"]
            .into_iter()
            .map(|o| {
                Quad::new(
                    terms::named_node("https://example.com/s").unwrap(),
                    terms::named_node("https://example.com/p").unwrap(),
                    Literal::new_simple_literal(o),
                    GraphName::DefaultGraph,
                )
            })
            .collect();
        model.bulk_insert_quads(quads).unwrap();
        let found: Vec<_> = model
            .find(StatementPattern {
                object: Some((&Literal::new_simple_literal("y")).into()),
                ..StatementPattern::default()
            })
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(found.len(), 1);
    }
}

#[test]
fn harness_fjall_sync_reopen_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("reopen");
    {
        let model = open_registered(StorageBackend::Fjall, &path);
        model.add(statement("persist")).unwrap();
        model.sync().unwrap();
    }
    let reopened = open_registered(StorageBackend::Fjall, &path);
    assert!(reopened.contains(statement("persist").as_ref()).unwrap());
}

#[test]
fn harness_wrong_backend_open_options_memory_ignores_path() {
    let dir = tempfile::tempdir().unwrap();
    let model = Model::open_with(OpenOptions::new(
        StorageBackend::Memory,
        dir.path().join("ignored"),
    ))
    .unwrap();
    assert_eq!(model.backend(), StorageBackend::Memory);
    assert!(!model.capabilities().durable);
}
