//! 0.6 safe-API accounting evidence for inventory families.

#[test]
fn inventory_accounting_families() {
    // Family classifications for ownership, hash/list (ADR-016), factories
    // (ADR-018), and excluded plugins are documented in
    // docs/design/0.6-safe-api-accounting.md and enforced by
    // scripts/check-inventory.py on redland-1.0.17-oxiland-0.6.json.
    let manifest = include_str!("../compatibility/inventory/redland-1.0.17-oxiland-0.6.json");
    assert!(manifest.contains("\"milestone\": \"0.6\""));
    assert!(!manifest.contains("\"state\": \"unreviewed\""));
    assert!(manifest.contains("not-applicable") || manifest.contains("excluded"));
}
