use std::path::Path;

use oxiland::io::{GraphTarget, Parser, Serializer, Syntax};
use oxiland::sparql::CancellationToken;
use oxiland::terms::{GraphName, Literal, NamedNode, NamedOrBlankNode, Triple};
use oxiland::{
    Error, Model, Query, QueryResults, ResultsFormat, Update, serialize_graph_results_to_writer,
    serialize_query_results_to_string,
};

fn load_alice(model: &Model) {
    model
        .add(Triple::new(
            NamedNode::new("https://example.com/alice").unwrap(),
            NamedNode::new("https://example.com/name").unwrap(),
            Literal::new_simple_literal("Alice"),
        ))
        .unwrap();
}

#[test]
fn ask_select_construct_describe_positive_paths() {
    let model = Model::new().unwrap();
    load_alice(&model);

    assert!(matches!(
        Query::new("ASK { ?s <https://example.com/name> ?o }")
            .execute(&model)
            .unwrap(),
        QueryResults::Boolean(true)
    ));

    let results = Query::new(
        "SELECT ?name WHERE { <https://example.com/alice> <https://example.com/name> ?name }",
    )
    .execute(&model)
    .unwrap();
    let QueryResults::Solutions(mut solutions) = results else {
        panic!("expected solutions");
    };
    let row = solutions.next().unwrap().unwrap();
    assert_eq!(
        row.get("name"),
        Some(&Literal::new_simple_literal("Alice").into())
    );
    assert!(solutions.next().is_none());

    let results = Query::new(
        "CONSTRUCT { ?s <https://example.com/label> ?o } WHERE { ?s <https://example.com/name> ?o }",
    )
    .execute(&model)
    .unwrap();
    let QueryResults::Graph(graph) = results else {
        panic!("expected graph");
    };
    let triples: Vec<_> = graph.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(triples.len(), 1);

    let results = Query::new("DESCRIBE <https://example.com/alice>")
        .execute(&model)
        .unwrap();
    assert!(matches!(results, QueryResults::Graph(_)));
}

#[test]
fn empty_ask_is_false() {
    let model = Model::new().unwrap();
    assert!(matches!(
        Query::new("ASK { ?s ?p ?o }").execute(&model).unwrap(),
        QueryResults::Boolean(false)
    ));
}

#[test]
fn empty_construct_yields_no_triples() {
    let model = Model::new().unwrap();
    let results = Query::new("CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }")
        .execute(&model)
        .unwrap();
    let QueryResults::Graph(graph) = results else {
        panic!("expected graph");
    };
    assert_eq!(graph.count(), 0);
}

#[test]
fn empty_select_yields_no_solutions() {
    let model = Model::new().unwrap();
    let results = Query::new("SELECT ?s WHERE { ?s ?p ?o }")
        .execute(&model)
        .unwrap();
    let QueryResults::Solutions(mut solutions) = results else {
        panic!("expected solutions");
    };
    assert!(solutions.next().is_none());
}

#[test]
fn parse_versus_evaluation_errors() {
    let model = Model::new().unwrap();
    let err = match Query::new("NOT SPARQL").execute(&model) {
        Err(error) => error,
        Ok(_) => panic!("expected parse error"),
    };
    assert!(matches!(err, Error::SparqlParse(_)));

    // Cooperative cancel surfaces as SparqlEvaluation while draining solutions.
    for i in 0..50 {
        model
            .add(Triple::new(
                NamedNode::new(format!("https://example.com/s{i}")).unwrap(),
                NamedNode::new("https://example.com/p").unwrap(),
                Literal::from(i),
            ))
            .unwrap();
    }
    let token = CancellationToken::new();
    let results = Query::new("SELECT ?s WHERE { ?s <https://example.com/p> ?o }")
        .cancellation_token(token.clone())
        .execute(&model)
        .unwrap();
    token.cancel();
    let err = serialize_query_results_to_string(results, ResultsFormat::Json).unwrap_err();
    assert!(matches!(err, Error::SparqlEvaluation(_)));
}

#[test]
fn limit_and_offset_slice_select() {
    let model = Model::new().unwrap();
    for i in 0..5 {
        model
            .add(Triple::new(
                NamedNode::new(format!("https://example.com/s{i}")).unwrap(),
                NamedNode::new("https://example.com/p").unwrap(),
                Literal::from(i),
            ))
            .unwrap();
    }
    let results = Query::new("SELECT ?s WHERE { ?s <https://example.com/p> ?o } ORDER BY ?s")
        .offset(1)
        .unwrap()
        .limit(2)
        .unwrap()
        .execute(&model)
        .unwrap();
    let QueryResults::Solutions(solutions) = results else {
        panic!("expected solutions");
    };
    assert_eq!(solutions.count(), 2);

    // API limit/offset replace in-query LIMIT/OFFSET rather than nesting Slice.
    let results = Query::new(
        "SELECT ?s WHERE { ?s <https://example.com/p> ?o } ORDER BY ?s LIMIT 100 OFFSET 0",
    )
    .limit(1)
    .unwrap()
    .execute(&model)
    .unwrap();
    let QueryResults::Solutions(solutions) = results else {
        panic!("expected solutions");
    };
    assert_eq!(solutions.count(), 1);

    let err = Query::new("ASK { ?s ?p ?o }").limit(1);
    assert!(matches!(err, Err(Error::Unsupported(_))));

    let err = Query::new("PREFIX ex: <https://example.com/> ASK { ?s ?p ?o }").limit(1);
    assert!(matches!(err, Err(Error::Unsupported(_))));

    let err = Query::new("\u{feff}ASK { ?s ?p ?o }").limit(1);
    assert!(matches!(err, Err(Error::Unsupported(_))));
}

#[test]
fn unbound_variables_are_none() {
    let model = Model::new().unwrap();
    load_alice(&model);
    let results = Query::new(
        "SELECT ?name ?missing WHERE { <https://example.com/alice> <https://example.com/name> ?name }",
    )
    .execute(&model)
    .unwrap();
    let QueryResults::Solutions(mut solutions) = results else {
        panic!("expected solutions");
    };
    let row = solutions.next().unwrap().unwrap();
    assert!(row.get("name").is_some());
    assert!(row.get("missing").is_none());
    assert!(row.get(1).is_none());
}

#[test]
fn dataset_default_graph_restricts_matches() {
    let model = Model::new().unwrap();
    let trig = r#"
        { <https://example.com/a> <https://example.com/p> "default" . }
        <https://example.com/g> { <https://example.com/a> <https://example.com/p> "named" . }
    "#;
    Parser::for_syntax(Syntax::TriG)
        .graph_target(GraphTarget::Dataset)
        .load_collecting(&model, trig.as_bytes())
        .unwrap();

    let g = GraphName::NamedNode(NamedNode::new("https://example.com/g").unwrap());
    let results =
        Query::new("SELECT ?o WHERE { <https://example.com/a> <https://example.com/p> ?o }")
            .default_graph([g])
            .execute(&model)
            .unwrap();
    let QueryResults::Solutions(mut solutions) = results else {
        panic!("expected solutions");
    };
    let row = solutions.next().unwrap().unwrap();
    assert_eq!(
        row.get("o"),
        Some(&Literal::new_simple_literal("named").into())
    );

    let results =
        Query::new("SELECT ?o WHERE { <https://example.com/a> <https://example.com/p> ?o }")
            .default_graph_as_union()
            .execute(&model)
            .unwrap();
    let QueryResults::Solutions(solutions) = results else {
        panic!("expected solutions");
    };
    assert_eq!(solutions.count(), 2);

    let named = NamedOrBlankNode::from(NamedNode::new("https://example.com/g").unwrap());
    let results = Query::new(
        "SELECT ?o WHERE { GRAPH <https://example.com/g> { <https://example.com/a> <https://example.com/p> ?o } }",
    )
    .available_named_graphs([named])
    .execute(&model)
    .unwrap();
    let QueryResults::Solutions(mut solutions) = results else {
        panic!("expected solutions");
    };
    assert_eq!(
        solutions.next().unwrap().unwrap().get("o"),
        Some(&Literal::new_simple_literal("named").into())
    );
}

#[test]
fn order_by_is_respected() {
    let model = Model::new().unwrap();
    for name in ["Carol", "Alice", "Bob"] {
        model
            .add(Triple::new(
                NamedNode::new(format!("https://example.com/{name}")).unwrap(),
                NamedNode::new("https://example.com/name").unwrap(),
                Literal::new_simple_literal(name),
            ))
            .unwrap();
    }
    let results =
        Query::new("SELECT ?name WHERE { ?s <https://example.com/name> ?name } ORDER BY ?name")
            .execute(&model)
            .unwrap();
    let QueryResults::Solutions(solutions) = results else {
        panic!("expected solutions");
    };
    let names: Vec<_> = solutions
        .map(|row| {
            let row = row.unwrap();
            row.get("name").unwrap().to_string()
        })
        .collect();
    assert_eq!(
        names,
        vec![
            "\"Alice\"".to_string(),
            "\"Bob\"".to_string(),
            "\"Carol\"".to_string()
        ]
    );
}

#[test]
fn select_and_construct_support_early_stop() {
    let model = Model::new().unwrap();
    for i in 0..200 {
        model
            .add(Triple::new(
                NamedNode::new(format!("https://example.com/s{i}")).unwrap(),
                NamedNode::new("https://example.com/p").unwrap(),
                Literal::from(i),
            ))
            .unwrap();
    }

    let results = Query::new("SELECT ?s WHERE { ?s <https://example.com/p> ?o }")
        .execute(&model)
        .unwrap();
    let QueryResults::Solutions(mut solutions) = results else {
        panic!("expected solutions");
    };
    let _ = solutions.next().unwrap().unwrap();
    drop(solutions); // early stop without collecting

    let results = Query::new("CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }")
        .execute(&model)
        .unwrap();
    let QueryResults::Graph(mut graph) = results else {
        panic!("expected graph");
    };
    let _ = graph.next().unwrap().unwrap();
    drop(graph);
}

#[test]
fn update_insert_delete_data() {
    let model = Model::new().unwrap();
    Update::new("INSERT DATA { <https://example.com/a> <https://example.com/p> \"x\" }")
        .execute(&model)
        .unwrap();
    assert_eq!(model.len().unwrap(), 1);

    Update::new("DELETE DATA { <https://example.com/a> <https://example.com/p> \"x\" }")
        .execute(&model)
        .unwrap();
    assert!(model.is_empty().unwrap());

    let err = Update::new("NOT UPDATE").execute(&model).unwrap_err();
    assert!(matches!(err, Error::SparqlParse(_)));

    let g = GraphName::NamedNode(NamedNode::new("https://example.com/g").unwrap());
    let err = Update::new("INSERT DATA { <https://example.com/a> <https://example.com/p> \"x\" }")
        .default_graph([g])
        .execute(&model)
        .unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)));

    Update::new("INSERT DATA { <https://example.com/a> <https://example.com/p> \"x\" }")
        .execute(&model)
        .unwrap();
    Update::new("DELETE { ?s ?p ?o } INSERT { ?s ?p \"y\" } WHERE { ?s ?p ?o }")
        .default_graph_as_union()
        .execute(&model)
        .unwrap();
    assert!(
        model
            .contains(
                Triple::new(
                    NamedNode::new("https://example.com/a").unwrap(),
                    NamedNode::new("https://example.com/p").unwrap(),
                    Literal::new_simple_literal("y"),
                )
                .as_ref()
            )
            .unwrap()
    );
}

#[test]
#[cfg(feature = "storage-fjall")]
fn fjall_update_persists() {
    let dir = tempfile::tempdir().unwrap();
    let model = Model::open(dir.path()).unwrap();
    Update::new("INSERT DATA { <https://example.com/a> <https://example.com/p> \"persist\" }")
        .execute(&model)
        .unwrap();
    drop(model);

    let reopened = Model::open(dir.path()).unwrap();
    assert_eq!(reopened.len().unwrap(), 1);
    assert!(
        reopened
            .contains(
                Triple::new(
                    NamedNode::new("https://example.com/a").unwrap(),
                    NamedNode::new("https://example.com/p").unwrap(),
                    Literal::new_simple_literal("persist"),
                )
                .as_ref()
            )
            .unwrap()
    );
}

#[test]
fn serialize_ask_and_select_results_formats() {
    let model = Model::new().unwrap();
    load_alice(&model);

    for name in ["xml", "json", "csv", "tsv"] {
        let format = ResultsFormat::from_name(name).unwrap();
        let ask = Query::new("ASK { ?s ?p ?o }").execute(&model).unwrap();
        let text = serialize_query_results_to_string(ask, format).unwrap();
        assert!(!text.is_empty(), "{name} ask empty");

        let select = Query::new(
            "SELECT ?name WHERE { <https://example.com/alice> <https://example.com/name> ?name }",
        )
        .execute(&model)
        .unwrap();
        let text = serialize_query_results_to_string(select, format).unwrap();
        assert!(
            text.contains("Alice") || text.contains("alice"),
            "{name}: {text}"
        );
    }

    assert!(ResultsFormat::from_name("yaml").is_err());
    assert_eq!(
        ResultsFormat::from_media_type("application/sparql-results+json; charset=utf-8").unwrap(),
        ResultsFormat::Json
    );

    let graph = Query::new("CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }")
        .execute(&model)
        .unwrap();
    let err = serialize_query_results_to_string(graph, ResultsFormat::Json).unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)));

    let results = Query::new(
        "CONSTRUCT { ?s <https://example.com/label> ?o } WHERE { ?s <https://example.com/name> ?o }",
    )
    .execute(&model)
    .unwrap();
    let turtle = String::from_utf8(
        serialize_graph_results_to_writer(
            results,
            &Serializer::for_syntax(Syntax::Turtle),
            Vec::new(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(turtle.contains("Alice"));
}

#[test]
fn cancellation_token_is_accepted() {
    let model = Model::new().unwrap();
    load_alice(&model);
    let token = CancellationToken::new();
    let results = Query::new("ASK { ?s ?p ?o }")
        .cancellation_token(token.clone())
        .execute(&model)
        .unwrap();
    assert!(matches!(results, QueryResults::Boolean(true)));

    Update::new("INSERT DATA { <https://example.com/b> <https://example.com/p> \"y\" }")
        .cancellation_token(token)
        .execute(&model)
        .unwrap();
}

#[test]
fn prefix_and_base_iri_configure_parsing() {
    let model = Model::new().unwrap();
    model
        .add(Triple::new(
            NamedNode::new("https://example.com/alice").unwrap(),
            NamedNode::new("https://example.com/name").unwrap(),
            Literal::new_simple_literal("Alice"),
        ))
        .unwrap();
    let results = Query::new("SELECT (ex:alice AS ?s) WHERE {}")
        .prefix("ex", "https://example.com/")
        .unwrap()
        .execute(&model)
        .unwrap();
    assert!(matches!(results, QueryResults::Solutions(_)));

    let err = match Query::new("SELECT * WHERE {}").base_iri("not a valid iri") {
        Err(error) => error,
        Ok(_) => panic!("expected InvalidRdf"),
    };
    assert!(matches!(err, Error::InvalidRdf(_)));

    let err =
        match Update::new("INSERT DATA { <https://example.com/a> <https://example.com/p> \"x\" }")
            .prefix("ex", ":::bad")
        {
            Err(error) => error,
            Ok(_) => panic!("expected InvalidRdf"),
        };
    assert!(matches!(err, Error::InvalidRdf(_)));
}

#[test]
fn query_results_debug_does_not_drain() {
    let model = Model::new().unwrap();
    load_alice(&model);
    let results = Query::new(
        "SELECT ?name WHERE { <https://example.com/alice> <https://example.com/name> ?name }",
    )
    .execute(&model)
    .unwrap();
    let debug = format!("{results:?}");
    assert!(debug.contains("Solutions"));
    let QueryResults::Solutions(mut solutions) = results else {
        panic!("expected solutions");
    };
    assert!(solutions.next().is_some());
}

#[test]
fn sparql_fixture_smoke() {
    let model = Model::new().unwrap();
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("compatibility/fixtures/sparql/smoke.ttl");
    Parser::for_syntax(Syntax::Turtle)
        .load_path_collecting(&model, path)
        .unwrap();
    assert!(matches!(
        Query::new("ASK { ?s <https://example.com/name> ?o }")
            .execute(&model)
            .unwrap(),
        QueryResults::Boolean(true)
    ));
}
