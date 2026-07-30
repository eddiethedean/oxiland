//! SELECT query example for Oxiland 0.2.

use oxiland::terms::{self, Literal, Triple};
use oxiland::{Model, Query, QueryResults};

fn main() -> oxiland::Result<()> {
    let model = Model::new()?;
    model.add(Triple::new(
        terms::named_node("https://example.com/alice")?,
        terms::named_node("https://example.com/name")?,
        Literal::new_simple_literal("Alice"),
    ))?;

    let results = Query::new(
        "SELECT ?name WHERE { <https://example.com/alice> <https://example.com/name> ?name }",
    )
    .execute(&model)?;

    match results {
        QueryResults::Solutions(solutions) => {
            for solution in solutions {
                let solution = solution
                    .map_err(|error| oxiland::Error::SparqlEvaluation(error.to_string()))?;
                if let Some(term) = solution.get("name") {
                    println!("name = {term}");
                }
            }
        }
        _other => panic!("expected SELECT solutions, got another QueryResults variant"),
    }

    Ok(())
}
