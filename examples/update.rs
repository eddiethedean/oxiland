//! SPARQL Update example for Oxiland 0.3.
//!
//! Run with `cargo run --example update`.

use oxiland::{Model, Query, QueryResults, Update};

fn main() -> oxiland::Result<()> {
    let model = Model::new()?;
    Update::new("INSERT DATA { <https://example.com/alice> <https://example.com/name> \"Alice\" }")
        .execute(&model)?;

    let results =
        Query::new("ASK { <https://example.com/alice> <https://example.com/name> \"Alice\" }")
            .execute(&model)?;
    assert!(matches!(results, QueryResults::Boolean(true)));

    Update::new("DELETE DATA { <https://example.com/alice> <https://example.com/name> \"Alice\" }")
        .execute(&model)?;
    assert!(model.is_empty()?);
    Ok(())
}
