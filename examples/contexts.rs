use oxiland::terms::{GraphName, Literal, NamedNode, Triple};
use oxiland::{Model, StatementPattern};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::new()?;
    let subject = NamedNode::new("https://example.com/alice")?;
    let graph = NamedNode::new("https://example.com/people")?;

    model.add_to_graph(
        Triple::new(
            subject.clone(),
            NamedNode::new("https://example.com/name")?,
            Literal::new_simple_literal("Alice"),
        ),
        GraphName::NamedNode(graph),
    )?;

    let matches = model.find(StatementPattern {
        subject: Some(subject.as_ref().into()),
        ..StatementPattern::default()
    })?;

    assert_eq!(matches.len(), 1);
    Ok(())
}
