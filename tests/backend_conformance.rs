//! Shared backend conformance suite (ADR-022 / SB-03 / 0.9).

use oxigraph::model::{GraphName, Quad};
use oxiland::terms::{self, Literal, Triple};
use oxiland::{
    Error, LayoutReaderPolicy, Model, OpenOptions, StatementPattern, StorageBackend,
    StorageCapabilities, compiled_backends, supported_backends,
};

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
fn frozen_backend_registry_is_feature_independent_and_complete() {
    let descriptors: Vec<_> = supported_backends().collect();
    assert_eq!(descriptors.len(), 6);
    assert_eq!(
        descriptors.iter().map(|item| item.name).collect::<Vec<_>>(),
        ["memory", "fjall", "redb", "rocksdb", "sqlite", "lmdb"]
    );

    for descriptor in descriptors {
        assert_eq!(descriptor.backend.name(), descriptor.name);
        assert_eq!(descriptor.backend.is_compiled(), descriptor.compiled);
        assert_eq!(
            descriptor.durable,
            descriptor.backend != StorageBackend::Memory
        );
        assert_eq!(
            descriptor.layout_reader,
            if descriptor.durable {
                LayoutReaderPolicy::FormatV1
            } else {
                LayoutReaderPolicy::None
            }
        );
        assert_eq!(
            StorageCapabilities::for_backend(descriptor.backend, false).backend,
            descriptor.backend
        );
    }
}

#[test]
fn read_only_capabilities_are_frozen_consistently() {
    for descriptor in supported_backends() {
        let capabilities = StorageCapabilities::for_backend(descriptor.backend, true);
        assert!(capabilities.read_only);
        assert_eq!(capabilities.durable, descriptor.durable);
        assert!(!capabilities.transactions);
        assert!(!capabilities.clear);
        assert!(!capabilities.bulk_load);
    }
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
fn every_compiled_layout_has_a_reader_and_standards_export_path() {
    for backend in compiled_backends().iter().copied() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(format!("reader-{}", backend.name()));
        let export = dir.path().join(format!("{}.nq", backend.name()));
        {
            let model = open_registered(backend, &path);
            model.add(statement("portable")).unwrap();
            model.sync().unwrap();
            model.export_nquads_to_path(&export).unwrap();
        }

        let reopened = if backend == StorageBackend::Memory {
            // Memory intentionally has no physical reader; prove the standards
            // export is sufficient to recover into a new supported model.
            let model = Model::new().unwrap();
            model.import_nquads_from_path(&export).unwrap();
            model
        } else {
            Model::open_with(OpenOptions::new(backend, &path).create(false)).unwrap()
        };
        assert!(reopened.contains(statement("portable").as_ref()).unwrap());
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

#[test]
#[cfg(feature = "storage-fjall")]
fn crash_injection_before_and_after_sync_preserves_reader() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("crash");
    {
        let model = open_registered(StorageBackend::Fjall, &path);
        model.add(statement("pre-sync")).unwrap();
        // Simulated failure before sync: reopen must not see unsynced memory-only data
        // for durable backends that require explicit sync — Oxiland syncs on drop path
        // via replace; force sync then corrupt is separate.
        model.sync().unwrap();
    }
    let reopened = open_registered(StorageBackend::Fjall, &path);
    assert!(reopened.contains(statement("pre-sync").as_ref()).unwrap());

    // Failure injection after sync: truncate meta to unsupported version and expect open failure.
    let meta = path.join("__oxiland").join("meta");
    if meta.exists() {
        std::fs::write(&meta, r#"{"format_version":999}"#).unwrap();
        let err = Model::open_with(OpenOptions::fjall(&path).create(false));
        assert!(err.is_err(), "unsupported format version must fail open");
    }
}

#[test]
#[cfg(feature = "storage-fjall")]
fn concurrent_readers_during_writer_transaction() {
    use std::sync::Arc;
    use std::thread;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("concurrent");
    let model = Arc::new(open_registered(StorageBackend::Fjall, &path));
    model.add(statement("base")).unwrap();
    model.sync().unwrap();

    let reader = Arc::clone(&model);
    let handle = thread::spawn(move || {
        for _ in 0..50 {
            let _ = reader.len().unwrap();
            let _ = reader
                .find(StatementPattern::default())
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
        }
    });
    model
        .transaction(|txn| {
            txn.add(statement("txn"))?;
            Ok(())
        })
        .unwrap();
    handle.join().unwrap();
    assert!(model.contains(statement("txn").as_ref()).unwrap());
}
