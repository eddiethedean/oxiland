use oxiland::terms::{self, GraphName, Literal, NamedNode, Triple};
use oxiland::{Error, Model, Query, QueryResults, StatementPattern, World};

fn example_statement() -> Triple {
    Triple::new(
        NamedNode::new("https://example.com/alice").unwrap(),
        NamedNode::new("https://example.com/name").unwrap(),
        Literal::new_simple_literal("Alice"),
    )
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn core_types_are_send_sync() {
    assert_send_sync::<Model>();
    assert_send_sync::<World>();
    assert_send_sync::<Query>();
}

#[test]
fn world_features_are_shared_across_clones() {
    let world = World::new();
    let clone = world.clone();
    world.set_feature(
        "http://example.com/feature",
        oxiland::FeatureValue::Boolean(true),
    );
    assert_eq!(
        clone.feature("http://example.com/feature"),
        Some(oxiland::FeatureValue::Boolean(true))
    );
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
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(matches.len(), 1);

    assert!(model.remove(statement).unwrap());
    assert!(model.is_empty().unwrap());
}

#[test]
fn model_clone_shares_dataset() {
    let model = Model::new().unwrap();
    let clone = model.clone();
    assert!(model.add(example_statement()).unwrap());
    assert_eq!(clone.len().unwrap(), 1);
}

#[test]
fn named_graph_crud_is_isolated_from_default_graph() {
    let model = Model::new().unwrap();
    let statement = example_statement();
    let graph = NamedNode::new("https://example.com/graph").unwrap();
    let graph_name = GraphName::NamedNode(graph.clone());

    assert!(
        model
            .add_to_graph(statement.clone(), graph_name.clone())
            .unwrap()
    );
    assert!(
        !model
            .add_to_graph(statement.clone(), graph_name.clone())
            .unwrap()
    );
    assert!(
        model
            .contains_in_graph(statement.as_ref(), graph_name.as_ref())
            .unwrap()
    );
    assert!(!model.contains(statement.as_ref()).unwrap());

    let in_context = model
        .find(StatementPattern {
            graph_name: Some(graph_name.as_ref()),
            ..StatementPattern::default()
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(in_context.len(), 1);

    let in_default = model
        .find(StatementPattern {
            graph_name: Some(oxiland::terms::GraphNameRef::DefaultGraph),
            ..StatementPattern::default()
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(in_default.is_empty());

    assert!(
        model
            .remove_from_graph(statement.clone(), graph_name.clone())
            .unwrap()
    );
    assert!(
        !model
            .contains_in_graph(statement.as_ref(), graph_name.as_ref())
            .unwrap()
    );
    assert!(model.is_empty().unwrap());
}

#[test]
fn find_streams_without_full_materialization() {
    let model = Model::new().unwrap();
    for i in 0..8 {
        model
            .add(Triple::new(
                NamedNode::new(format!("https://example.com/s/{i}")).unwrap(),
                NamedNode::new("https://example.com/name").unwrap(),
                Literal::new_simple_literal(format!("v{i}")),
            ))
            .unwrap();
    }

    let first = model
        .find(StatementPattern::default())
        .next()
        .unwrap()
        .unwrap();
    assert!(first.subject.to_string().contains("example.com/s/"));
}

#[test]
fn model_supports_contexts_and_sparql() {
    let model = Model::new().unwrap();
    let graph = NamedNode::new("https://example.com/graph").unwrap();
    model
        .add_to_graph(example_statement(), GraphName::NamedNode(graph))
        .unwrap();

    let ask = Query::new("ASK WHERE { GRAPH ?g { ?s ?p ?o } }")
        .execute(&model)
        .unwrap();
    assert!(matches!(ask, QueryResults::Boolean(true)));

    let QueryResults::Solutions(mut solutions) =
        Query::new("SELECT ?name WHERE { GRAPH ?g { ?s <https://example.com/name> ?name } }")
            .execute(&model)
            .unwrap()
    else {
        panic!("expected solution results");
    };
    let solution = solutions.next().unwrap().unwrap();
    assert_eq!(
        solution.get("name").map(ToString::to_string).as_deref(),
        Some("\"Alice\"")
    );
}

#[test]
fn invalid_iri_maps_to_invalid_rdf_error() {
    let error = terms::named_node("not a valid iri").unwrap_err();
    assert!(matches!(error, Error::InvalidRdf(_)));
}

#[test]
fn invalid_blank_node_id_maps_to_invalid_rdf_error() {
    let error = terms::blank_node(Some("bad id")).unwrap_err();
    assert!(matches!(error, Error::InvalidRdf(_)));
}

#[test]
fn sparql_parse_errors_are_distinct_from_evaluation() {
    let model = Model::new().unwrap();
    match Query::new("NOT VALID SPARQL").execute(&model) {
        Err(Error::SparqlParse(_)) => {}
        Err(error) => panic!("expected SparqlParse, got {error}"),
        Ok(_) => panic!("expected SparqlParse, got successful query results"),
    }
}

#[test]
fn unsupported_storage_backend_is_explicit() {
    let error = Model::storage_backend_available("mysql").unwrap_err();
    assert!(matches!(error, Error::Unsupported(_)));
    assert!(Model::storage_backend_available("memory").unwrap());
}

#[cfg(feature = "rocksdb")]
#[test]
fn rocksdb_model_round_trips_statements() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store");
    {
        let model = Model::open(&path).unwrap();
        assert!(model.add(example_statement()).unwrap());
        assert_eq!(model.len().unwrap(), 1);
    }
    let reopened = Model::open(&path).unwrap();
    assert!(reopened.contains(example_statement().as_ref()).unwrap());
    assert!(Model::storage_backend_available("rocksdb").unwrap());
}
