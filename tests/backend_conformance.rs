//! Shared backend conformance suite (ADR-022 / SB-03 / 0.9).

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
    Model::open_with(OpenOptions::new(backend, path).create(true)).unwrap()
}

#[test]
fn compiled_backends_include_memory_and_enabled_fjall() {
    let backends = compiled_backends();
    assert!(backends.contains(&StorageBackend::Memory));
    #[cfg(feature = "storage-fjall")]
    assert!(backends.contains(&StorageBackend::Fjall));
    #[cfg(not(feature = "storage-fjall"))]
    assert!(!backends.contains(&StorageBackend::Fjall));
}

#[test]
fn registry_rejects_unknown() {
    let err = StorageBackend::from_name("not-a-backend").unwrap_err();
    assert!(matches!(err, Error::Unsupported(msg) if msg.contains("not recognized")));
}

#[test]
fn registry_rejects_evaluation_backends_as_uncompiled() {
    let err = StorageBackend::from_name("sled").unwrap_err();
    assert!(matches!(err, Error::Unsupported(msg) if msg.contains("not compiled")));
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
fn harness_durable_sync_reopen_survives_restart() {
    for backend in compiled_backends()
        .iter()
        .copied()
        .filter(|b| *b != StorageBackend::Memory)
    {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(format!("reopen-{}", backend.name()));
        {
            let model = open_registered(backend, &path);
            model.add(statement("persist")).unwrap();
            model.sync().unwrap();
        }
        let reopened = Model::open_with(OpenOptions::new(backend, &path).create(false)).unwrap();
        assert!(reopened.contains(statement("persist").as_ref()).unwrap());
    }
}

#[test]
#[cfg(feature = "storage-fjall")]
fn wrong_backend_open_fails_before_mutation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("owned");
    {
        let model = open_registered(StorageBackend::Fjall, &path);
        model.add(statement("keep")).unwrap();
        model.sync().unwrap();
    }
    // Opening the Fjall layout as another compiled backend must fail.
    for backend in compiled_backends()
        .iter()
        .copied()
        .filter(|b| !matches!(b, StorageBackend::Memory | StorageBackend::Fjall))
    {
        let err = match Model::open_with(OpenOptions::new(backend, &path).create(true)) {
            Ok(_) => panic!("expected wrong-backend open to fail for {:?}", backend),
            Err(err) => err,
        };
        assert!(
            matches!(err, Error::OpenStore { .. }),
            "expected OpenStore for {:?}, got {err:?}",
            backend
        );
    }
    let still =
        Model::open_with(OpenOptions::new(StorageBackend::Fjall, &path).create(false)).unwrap();
    assert!(still.contains(statement("keep").as_ref()).unwrap());
}

#[test]
#[cfg(feature = "storage-fjall")]
fn copy_to_memory_and_durable() {
    let src = Model::new().unwrap();
    src.add(statement("copy-me")).unwrap();
    let mem = src
        .copy_to(OpenOptions::new(StorageBackend::Memory, "/tmp/unused"))
        .unwrap();
    assert!(mem.contains(statement("copy-me").as_ref()).unwrap());

    let dir = tempfile::tempdir().unwrap();
    let dest_path = dir.path().join("dest-fjall");
    let dest = src
        .copy_to(OpenOptions::fjall(&dest_path).create(true))
        .unwrap();
    assert_eq!(dest.backend(), StorageBackend::Fjall);
    assert!(dest.contains(statement("copy-me").as_ref()).unwrap());
}
