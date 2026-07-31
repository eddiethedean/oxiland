use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn bin() -> Command {
    Command::cargo_bin("oxiland-cli").expect("oxiland-cli binary")
}

#[test]
fn parse_find_query_round_trip() {
    let dir = tempdir().unwrap();
    let ttl = dir.path().join("data.ttl");
    fs::write(
        &ttl,
        r#"@prefix ex: <https://example.com/> .
ex:alice ex:name "Alice" .
"#,
    )
    .unwrap();

    let store = dir.path().join("store");
    bin()
        .args([
            "-n",
            "-s",
            "fjall",
            store.to_str().unwrap(),
            "parse",
            ttl.to_str().unwrap(),
            "--syntax",
            "turtle",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("parsed"));

    bin()
        .args([
            "-s",
            "fjall",
            store.to_str().unwrap(),
            "find",
            "-",
            "-",
            "-",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));

    bin()
        .args([
            "-s",
            "fjall",
            "-r",
            "csv",
            store.to_str().unwrap(),
            "query",
            "-",
            "-",
            "SELECT ?name WHERE { ?s <https://example.com/name> ?name }",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));
}

#[test]
fn named_graph_print_defaults_to_nquads() {
    let dir = tempdir().unwrap();
    let store = dir.path().join("named");
    bin()
        .args([
            "-n",
            "-s",
            "fjall",
            store.to_str().unwrap(),
            "add",
            "https://example.com/s",
            "https://example.com/p",
            "literal",
            "https://example.com/g",
        ])
        .assert()
        .success();

    bin()
        .args(["-s", "fjall", store.to_str().unwrap(), "print"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://example.com/g"));

    bin()
        .args([
            "-s",
            "fjall",
            store.to_str().unwrap(),
            "find",
            "-",
            "-",
            "-",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("literal"));
}

#[test]
fn create_false_missing_path_fails_without_new() {
    let dir = tempdir().unwrap();
    let store = dir.path().join("missing");
    bin()
        .args(["-s", "fjall", store.to_str().unwrap(), "print"])
        .assert()
        .failure();
}

#[test]
fn unsupported_storage_fails() {
    bin()
        .args(["-s", "mysql", "db", "print"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsupported storage"));
    bin()
        .args(["-s", "hashes", "db", "print"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsupported storage"));
}

#[test]
fn unsupported_query_language_fails() {
    bin()
        .args([
            "-s",
            "memory",
            "memory",
            "query",
            "rdql",
            "-",
            "SELECT ?s WHERE { ?s ?p ?o }",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsupported query language"));
}

#[test]
fn typed_literal_cli_arg_rejected() {
    bin()
        .args([
            "-s",
            "memory",
            "memory",
            "add",
            "https://example.com/s",
            "https://example.com/p",
            "\"42\"^^<http://www.w3.org/2001/XMLSchema#integer>",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("typed/language-tagged"));
}

#[test]
fn fjall_store_named_memory_rejected() {
    bin()
        .args(["-s", "fjall", "memory", "print"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires -s memory"));
}

#[test]
fn version_flag_works() {
    bin().arg("--version").assert().success();
}
