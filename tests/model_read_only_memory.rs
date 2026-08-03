//! Regression: memory OpenOptions must honor read_only.
use oxiland::terms::{Literal, Triple, named_node};
use oxiland::{Model, OpenOptions, StatementPattern, StorageBackend};

#[test]
fn memory_read_only_rejects_writes() {
    let model = Model::open_with(OpenOptions::new(StorageBackend::Memory, ".").read_only(true))
        .expect("open memory read-only");
    assert!(model.capabilities().read_only);
    let err = model
        .add(Triple::new(
            named_node("http://ex/s").unwrap(),
            named_node("http://ex/p").unwrap(),
            Literal::new_simple_literal("o"),
        ))
        .expect_err("write must fail");
    let _ = err;
    assert_eq!(model.find(StatementPattern::default()).count(), 0);
}
