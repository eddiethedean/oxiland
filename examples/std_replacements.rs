//! Redland hashes/lists map to standard Rust collections (ADR-016).

use std::collections::HashMap;

use oxiland::Model;
use oxiland::terms::{self, Literal, Triple};
use oxiland::utility::vocab::rdf;

fn main() -> oxiland::Result<()> {
    // Redland "list of statements" → Vec / iterator
    let model = Model::new()?;
    let statements = vec![
        Triple::new(
            terms::named_node("https://example.com/alice")?,
            rdf::type_(),
            terms::named_node("https://example.com/Person")?,
        ),
        Triple::new(
            terms::named_node("https://example.com/alice")?,
            terms::named_node("https://example.com/name")?,
            Literal::new_simple_literal("Alice"),
        ),
    ];
    for statement in &statements {
        model.add(statement.clone())?;
    }

    // Redland "hash of URI → node" → HashMap
    let mut by_name = HashMap::new();
    by_name.insert("alice", terms::named_node("https://example.com/alice")?);
    println!(
        "mapped {} statements; alice = {}",
        statements.len(),
        by_name["alice"]
    );
    Ok(())
}
