use oxiland::terms::{GraphName, Literal, NamedNode, Triple};
use oxiland::{Model, Query, QueryResults, StatementPattern, World};

fn example_statement() -> Triple {
    Triple::new(
        NamedNode::new("https://example.com/alice").unwrap(),
        NamedNode::new("https://example.com/name").unwrap(),
        Literal::new_simple_literal("Alice"),
    )
}

#[test]
fn world_is_ready_on_construction() {
    let _world = World::new();
}

#[test]
fn model_supports_redland_style_crud_and_matching() {
    let model = Model::new().unwrap();
    let statement = example_statement();

    assert!(model.add(statement.clone()).unwrap());
    assert!(!model.add(statement.clone()).unwrap());
    assert!(model.contains(statement.as_ref()).unwrap());
    assert_eq!(model.len().unwrap(), 1);

    let matches = model
        .find(StatementPattern {
            subject: Some(statement.subject.as_ref()),
            ..StatementPattern::default()
        })
        .unwrap();
    assert_eq!(matches.len(), 1);

    assert!(model.remove(statement).unwrap());
    assert!(model.is_empty().unwrap());
}

#[test]
fn model_supports_contexts_and_sparql() {
    let model = Model::new().unwrap();
    let graph = NamedNode::new("https://example.com/graph").unwrap();
    model
        .add_to_graph(example_statement(), GraphName::NamedNode(graph))
        .unwrap();

    let results = Query::new("ASK WHERE { GRAPH ?g { ?s ?p ?o } }")
        .execute(&model)
        .unwrap();
    assert!(matches!(results, QueryResults::Boolean(true)));
}
