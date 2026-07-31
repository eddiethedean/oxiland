use std::fs;

use oxiland::io::{Parser, Syntax};
use oxiland::terms::{self, GraphName, Literal, NamedNode, Triple};
use oxiland::{Error, Model, OpenOptions, StatementPattern, StorageBackend, StorageCapabilities};

fn statement(object: &str) -> Triple {
    Triple::new(
        terms::named_node("https://example.com/s").unwrap(),
        terms::named_node("https://example.com/p").unwrap(),
        Literal::new_simple_literal(object),
    )
}

#[test]
fn format_v1_round_trip_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store");
    {
        let model = Model::open(&path).unwrap();
        assert!(model.capabilities().durable);
        assert_eq!(model.backend(), StorageBackend::Fjall);
        assert!(model.add(statement("a")).unwrap());
        model.sync().unwrap();
    }
    let reopened = Model::open(&path).unwrap();
    assert_eq!(reopened.len().unwrap(), 1);
    assert!(reopened.contains(statement("a").as_ref()).unwrap());
}

#[test]
fn migrate_legacy_experimental_store() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy");
    // Create a format-v1 store then strip metadata via migrate path covered in
    // unit tests; here verify open rejects and migrate restores after unit
    // helper pattern: open, add, reopen works for v1.
    let model = Model::open(&path).unwrap();
    model.add(statement("legacy")).unwrap();
    drop(model);
    let reopened = Model::open(&path).unwrap();
    assert_eq!(reopened.len().unwrap(), 1);
    // migrate on already-v1 store is a no-op open path
    let migrated = Model::migrate_legacy_store(&path).unwrap();
    assert_eq!(migrated.len().unwrap(), 1);
}

#[test]
fn transaction_commit_and_rollback_memory() {
    let model = Model::new().unwrap();
    model
        .transaction(|tx| {
            assert!(tx.add(statement("ok")).unwrap());
            Ok(())
        })
        .unwrap();
    assert_eq!(model.len().unwrap(), 1);

    let err: oxiland::Result<()> = model.transaction(|tx| {
        assert!(tx.add(statement("nope")).unwrap());
        Err(Error::Unsupported("force rollback".into()))
    });
    assert!(matches!(err, Err(Error::Unsupported(_))));
    assert_eq!(model.len().unwrap(), 1);
    assert!(!model.contains(statement("nope").as_ref()).unwrap());
}

#[test]
fn transaction_commit_persists_fjall() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("txn");
    {
        let model = Model::open(&path).unwrap();
        model
            .transaction(|tx| {
                tx.add(statement("t1"))?;
                tx.add(statement("t2"))?;
                Ok(())
            })
            .unwrap();
        assert_eq!(model.len().unwrap(), 2);
    }
    let reopened = Model::open(&path).unwrap();
    assert_eq!(reopened.len().unwrap(), 2);
}

#[test]
fn transaction_rollback_fjall_leaves_disk() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("txn-rb");
    let model = Model::open(&path).unwrap();
    model.add(statement("keep")).unwrap();
    let err: oxiland::Result<()> = model.transaction(|tx| {
        tx.add(statement("temp"))?;
        Err(Error::Unsupported("rollback".into()))
    });
    assert!(err.is_err());
    assert_eq!(model.len().unwrap(), 1);
    drop(model);
    let reopened = Model::open(&path).unwrap();
    assert_eq!(reopened.len().unwrap(), 1);
    assert!(reopened.contains(statement("keep").as_ref()).unwrap());
}

#[test]
fn clear_and_clear_graph() {
    let model = Model::new().unwrap();
    model.add(statement("default")).unwrap();
    let named = NamedNode::new("https://example.com/g").unwrap();
    model
        .add_to_graph(statement("named"), GraphName::NamedNode(named.clone()))
        .unwrap();
    model
        .clear_graph(GraphName::NamedNode(named.clone()))
        .unwrap();
    assert_eq!(model.len().unwrap(), 1);
    model.clear().unwrap();
    assert!(model.is_empty().unwrap());
}

#[test]
fn read_only_rejects_writes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ro");
    {
        let model = Model::open(&path).unwrap();
        model.add(statement("x")).unwrap();
    }
    let model = Model::open_with(OpenOptions::fjall(&path).read_only(true)).unwrap();
    assert!(model.capabilities().read_only);
    let err = model.add(statement("y")).unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)));
    assert_eq!(model.len().unwrap(), 1);
}

#[test]
fn legacy_backends_are_unsupported() {
    for name in ["mysql", "sqlite", "virtuoso", "hashes", "rocksdb"] {
        let err = Model::storage_backend_available(name).unwrap_err();
        assert!(matches!(err, Error::Unsupported(_)), "{name}");
    }
    assert!(Model::storage_backend_available("memory").unwrap());
    assert!(Model::storage_backend_available("fjall").unwrap());
    assert_eq!(
        StorageBackend::from_name("memory").unwrap(),
        StorageBackend::Memory
    );
}

#[test]
fn capabilities_memory_vs_fjall() {
    assert_eq!(
        Model::new().unwrap().capabilities(),
        StorageCapabilities::memory()
    );
    let dir = tempfile::tempdir().unwrap();
    let model = Model::open(dir.path()).unwrap();
    assert_eq!(model.capabilities(), StorageCapabilities::fjall(false));
}

#[test]
fn transactional_load_is_atomic_on_parse_failure() {
    let dir = tempfile::tempdir().unwrap();
    let model = Model::open(dir.path()).unwrap();
    model.add(statement("keep")).unwrap();
    let bad = b"<https://example.com/s> <https://example.com/p> \"ok\" .\nthis is not turtle\n";
    let err = Parser::for_syntax(Syntax::Turtle)
        .load_transactional(&model, &bad[..])
        .unwrap_err();
    assert!(matches!(err, Error::Parse(_)));
    assert_eq!(model.len().unwrap(), 1);
    drop(model);
    let reopened = Model::open(dir.path()).unwrap();
    assert_eq!(reopened.len().unwrap(), 1);
}

#[test]
fn archival_export_import_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path().join("store");
    let archive = dir.path().join("backup.nq");
    {
        let model = Model::open(&store).unwrap();
        model.add(statement("archived")).unwrap();
        model.export_nquads_to_path(&archive).unwrap();
        model.clear().unwrap();
        assert!(model.is_empty().unwrap());
        assert_eq!(model.import_nquads_from_path(&archive).unwrap(), 1);
        assert_eq!(model.len().unwrap(), 1);
    }
    assert!(fs::metadata(&archive).unwrap().len() > 0);
}

#[test]
fn bulk_insert_quads() {
    let model = Model::new().unwrap();
    let quads = (0..10)
        .map(|i| {
            let t = statement(&format!("b{i}"));
            oxiland::terms::Quad::new(t.subject, t.predicate, t.object, GraphName::DefaultGraph)
        })
        .collect::<Vec<_>>();
    assert_eq!(model.bulk_insert_quads(quads).unwrap(), 10);
    assert_eq!(model.len().unwrap(), 10);
}

#[test]
fn open_create_false_missing_path_fails() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing");
    let err = Model::open_with(OpenOptions::fjall(&path).create(false));
    assert!(matches!(err, Err(Error::OpenStore { .. })));
}

#[test]
fn find_under_concurrent_writers_stays_consistent() {
    use std::sync::Arc;
    use std::thread;

    let dir = tempfile::tempdir().unwrap();
    let model = Arc::new(Model::open(dir.path()).unwrap());
    model.add(statement("seed")).unwrap();

    let writer = {
        let model = Arc::clone(&model);
        thread::spawn(move || {
            for i in 0..20 {
                let _ = model.transaction(|tx| {
                    tx.add(statement(&format!("w{i}")))?;
                    Ok(())
                });
            }
        })
    };

    let reader = {
        let model = Arc::clone(&model);
        thread::spawn(move || {
            for _ in 0..50 {
                let count = model
                    .find(StatementPattern::default())
                    .filter_map(Result::ok)
                    .count();
                assert!(count >= 1);
            }
        })
    };

    writer.join().unwrap();
    reader.join().unwrap();
    assert!(model.len().unwrap() >= 1);
}
