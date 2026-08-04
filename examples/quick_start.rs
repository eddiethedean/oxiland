use oxiland::terms::{Literal, Triple, named_node};
use oxiland::{Model, Query, QueryResults};

fn main() -> oxiland::Result<()> {
    let model = Model::new()?;

    model.add(Triple::new(
        named_node("https://example.com/alice")?,
        named_node("https://example.com/name")?,
        Literal::new_simple_literal("Alice"),
    ))?;

    let result = Query::new("ASK { ?s ?p ?o }").execute(&model)?;
    assert!(matches!(result, QueryResults::Boolean(true)));

    Ok(())
}
