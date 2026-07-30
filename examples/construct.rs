//! CONSTRUCT query example for Oxiland 0.3.
//!
//! Run with `cargo run --example construct`.

use oxiland::io::{Serializer, Syntax};
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
        "CONSTRUCT { ?s <https://example.com/label> ?o } WHERE { ?s <https://example.com/name> ?o }",
    )
    .execute(&model)?;

    let out = Model::new()?;
    if let QueryResults::Graph(graph) = results {
        for triple in graph {
            out.add(triple.map_err(|error| oxiland::Error::SparqlEvaluation(error.to_string()))?)?;
        }
    }

    let turtle = Serializer::for_syntax(Syntax::Turtle).serialize_model_to_string(&out)?;
    print!("{turtle}");
    Ok(())
}
