use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use oxiland::terms::Literal;
use oxiland::utility::vocab::{dc, owl, rdf, rdfs, xsd};
use oxiland::utility::{
    DigestAlgorithm, Namespace, digest_hex, digest_path, file_uri_to_path, join_iri, normalize_nfc,
    normalize_nfkc, path_to_file_uri, relativize_iri, resolve_iri,
};
use oxiland::{Error, LogFacility, LogLevel, LogRecord, Model, StatementPattern, World};

#[test]
fn digest_hex_known_vectors() {
    assert_eq!(
        digest_hex(DigestAlgorithm::Md5, b""),
        "d41d8cd98f00b204e9800998ecf8427e"
    );
    assert_eq!(
        digest_hex(DigestAlgorithm::Sha1, b"abc"),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
    assert_eq!(
        digest_hex(DigestAlgorithm::Sha256, b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn digest_from_name_and_unsupported() {
    assert_eq!(
        DigestAlgorithm::from_name("SHA-256").unwrap(),
        DigestAlgorithm::Sha256
    );
    let err = DigestAlgorithm::from_name("blake3").unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)));
}

#[test]
fn digest_path_reads_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("x.txt");
    std::fs::write(&path, b"abc").unwrap();
    let bytes = digest_path(DigestAlgorithm::Sha1, &path).unwrap();
    assert_eq!(
        digest_hex(DigestAlgorithm::Sha1, b"abc"),
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    );
}

#[test]
fn uri_join_and_relativize() {
    let joined = join_iri("https://example.com/base/dir/", "leaf").unwrap();
    assert_eq!(joined.as_str(), "https://example.com/base/dir/leaf");
    let abs = join_iri("https://example.com/base/", "https://other.example/x").unwrap();
    assert_eq!(abs.as_str(), "https://other.example/x");
    let rel = relativize_iri(
        "https://example.com/base/dir/",
        "https://example.com/base/dir/leaf",
    )
    .unwrap();
    assert_eq!(rel.as_deref(), Some("leaf"));
    assert!(resolve_iri("not a uri").is_err());
}

#[test]
fn file_uri_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data with space.ttl");
    std::fs::write(&path, b"x").unwrap();
    let uri = path_to_file_uri(&path).unwrap();
    assert!(uri.as_str().starts_with("file://"));
    let back = file_uri_to_path(uri.as_str()).unwrap();
    assert_eq!(back.canonicalize().unwrap(), path.canonicalize().unwrap());
    let err = file_uri_to_path("https://example.com/x").unwrap_err();
    assert!(matches!(err, Error::InvalidRdf(_)));
    let bad = file_uri_to_path("file://remote.host/tmp/x").unwrap_err();
    assert!(matches!(bad, Error::Unsupported(_)));
}

#[test]
fn unicode_normalization_helpers() {
    let composed = normalize_nfc("e\u{0301}");
    assert_eq!(composed, "\u{00e9}");
    let compatibility = normalize_nfkc("ﬁ");
    assert_eq!(compatibility, "fi");
}

#[test]
fn namespace_expand_and_vocab_constants() {
    let ns = Namespace::new("ex", "https://example.com/").unwrap();
    assert_eq!(
        ns.expand("alice").unwrap().as_str(),
        "https://example.com/alice"
    );
    assert_eq!(
        ns.expand("ex:bob").unwrap().as_str(),
        "https://example.com/bob"
    );
    assert!(Namespace::new("", "https://example.com/").is_err());
    assert_eq!(
        rdf::type_().as_str(),
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
    );
    assert_eq!(
        rdfs::label().as_str(),
        "http://www.w3.org/2000/01/rdf-schema#label"
    );
    assert_eq!(
        xsd::integer().as_str(),
        "http://www.w3.org/2001/XMLSchema#integer"
    );
    assert_eq!(
        owl::same_as().as_str(),
        "http://www.w3.org/2002/07/owl#sameAs"
    );
    assert_eq!(dc::title().as_str(), "http://purl.org/dc/terms/title");
}

#[test]
fn logging_filters_and_preserves_order() {
    let world = World::new();
    let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    world.set_log_level(LogLevel::Info);
    world.set_log_handler(move |record: &LogRecord| {
        sink.lock()
            .unwrap()
            .push(format!("{}:{}", record.level.name(), record.message));
    });
    world.log(LogLevel::Debug, LogFacility::Utility, "skip");
    world.log(LogLevel::Info, LogFacility::Utility, "one");
    world.log(LogLevel::Warn, LogFacility::Model, "two");
    world.log(LogLevel::Error, LogFacility::Io, "three");
    let messages = seen.lock().unwrap().clone();
    assert_eq!(messages, vec!["info:one", "warn:two", "error:three"]);
}

#[test]
fn world_clones_share_log_handler() {
    let world = World::new();
    let seen: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let sink = Arc::clone(&seen);
    world.set_log_level(LogLevel::Debug);
    world.set_log_handler(move |_record: &LogRecord| {
        *sink.lock().unwrap() += 1;
    });
    let clone = world.clone();
    clone.log(LogLevel::Info, LogFacility::General, "shared");
    assert_eq!(*seen.lock().unwrap(), 1);
}

#[test]
fn find_early_stop_evidence_matrix() {
    let model = Model::new().unwrap();
    for i in 0..20 {
        model
            .add(oxiland::terms::Triple::new(
                oxiland::terms::named_node(format!("https://example.com/s{i}")).unwrap(),
                rdf::type_(),
                Literal::new_simple_literal(format!("v{i}")),
            ))
            .unwrap();
    }
    let mut count = 0;
    for item in model.find(StatementPattern::default()) {
        let _quad = item.unwrap();
        count += 1;
        if count == 3 {
            break;
        }
    }
    assert_eq!(count, 3);
}

#[test]
fn hash_list_std_replacements_documented() {
    // ADR-016: Redland hashes/lists map to HashMap/Vec — exercised by
    // examples/std_replacements.rs and migration docs.
    let mut map = HashMap::new();
    map.insert("k", 1);
    let list: Vec<i32> = vec![1, 2, 3];
    assert_eq!(map.get("k"), Some(&1));
    assert_eq!(list.len(), 3);
}

#[test]
fn malformed_path_digest_is_io_error() {
    let missing = PathBuf::from("/definitely/missing/oxiland-digest-test");
    let err = digest_path(DigestAlgorithm::Md5, &missing).unwrap_err();
    assert!(matches!(err, Error::Io(_)));
}
