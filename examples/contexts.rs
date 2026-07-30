use oxiland::terms::{GraphName, Literal, NamedNode, Triple};
use oxiland::{Model, StatementPattern};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::new()?;
    let subject = NamedNode::new("https://example.com/alice")?;
    let graph = NamedNode::new("https://example.com/people")?;
    let graph_name = GraphName::NamedNode(graph);

    let statement = Triple::new(
        subject.clone(),
        NamedNode::new("https://example.com/name")?,
        Literal::new_simple_literal("Alice"),
    );

    model.add_to_graph(statement.clone(), graph_name.clone())?;
    assert!(model.contains_in_graph(statement.as_ref(), graph_name.as_ref())?);
    assert!(!model.contains(statement.as_ref())?);

    let matches = model
        .find(StatementPattern {
            subject: Some(subject.as_ref().into()),
            ..StatementPattern::default()
        })
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(matches.len(), 1);
    assert!(model.remove_from_graph(statement, graph_name)?);
    Ok(())
}
