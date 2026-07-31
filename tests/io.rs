use std::io::{Cursor, Read};

use oxiland::io::{GraphTarget, Parser, Serializer, Syntax};
use oxiland::terms::{self, GraphName, Literal, NamedNode, Quad, Triple};
use oxiland::{Error, Model};

fn alice_triple() -> Triple {
    Triple::new(
        NamedNode::new("https://example.com/alice").unwrap(),
        NamedNode::new("https://example.com/name").unwrap(),
        Literal::new_simple_literal("Alice"),
    )
}

#[test]
fn syntax_lookup_covers_names_media_types_and_extensions() {
    let cases = [
        (Syntax::Turtle, "turtle", "text/turtle", "ttl"),
        (Syntax::NTriples, "ntriples", "application/n-triples", "nt"),
        (Syntax::NQuads, "nquads", "application/n-quads", "nq"),
        (Syntax::TriG, "trig", "application/trig", "trig"),
        (Syntax::RdfXml, "rdfxml", "application/rdf+xml", "rdf"),
    ];
    for (syntax, name, media, ext) in cases {
        assert_eq!(Syntax::from_name(name).unwrap(), syntax);
        assert_eq!(Syntax::from_media_type(media).unwrap(), syntax);
        assert_eq!(Syntax::from_extension(ext).unwrap(), syntax);
        assert_eq!(syntax.name(), name);
        assert_eq!(syntax.media_type(), media);
        assert_eq!(syntax.extension(), ext);
        assert!(syntax.can_parse());
        assert!(syntax.can_serialize());
    }
    assert!(Syntax::from_name("jsonld").is_err());
    assert!(Syntax::from_name("n3").is_err());
    assert!(Syntax::from_name("unknown").is_err());
    assert!(matches!(
        Syntax::from_media_type("text/plain"),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        Syntax::from_media_type("application/xml"),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        Syntax::from_extension("txt"),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        Syntax::from_extension("xml"),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn round_trip_each_advertised_syntax() {
    let triple = alice_triple();
    for syntax in Syntax::all() {
        let model = Model::new().unwrap();
        model.add(triple.clone()).unwrap();
        let serialized = Serializer::for_syntax(*syntax)
            .serialize_model_to_string(&model)
            .unwrap();
        let reloaded = Model::new().unwrap();
        let count = Parser::for_syntax(*syntax)
            .load_collecting(&reloaded, serialized.as_bytes())
            .unwrap();
        assert_eq!(count, 1, "syntax {}", syntax.name());
        assert!(reloaded.contains(triple.as_ref()).unwrap());
    }
}

#[test]
fn parser_streams_and_supports_early_stop() {
    use std::cell::Cell;
    use std::rc::Rc;

    struct CountingReader {
        inner: Cursor<Vec<u8>>,
        bytes_read: Rc<Cell<usize>>,
    }
    impl Read for CountingReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let n = self.inner.read(buf)?;
            self.bytes_read.set(self.bytes_read.get() + n);
            Ok(n)
        }
    }

    let mut input = Vec::new();
    for i in 0..20_000 {
        let line = format!("<https://example.com/{i}> <https://example.com/p> \"v\" .\n");
        input.extend_from_slice(line.as_bytes());
    }
    let total = input.len();
    let bytes_read = Rc::new(Cell::new(0usize));
    let reader = CountingReader {
        inner: Cursor::new(input),
        bytes_read: Rc::clone(&bytes_read),
    };
    {
        let mut stream = Parser::for_syntax(Syntax::NTriples)
            .parse_reader(reader)
            .unwrap();
        let first = stream.next().unwrap().unwrap();
        assert_eq!(
            first.subject,
            terms::named_node("https://example.com/0").unwrap().into()
        );
        let more: Vec<_> = stream.take(3).collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(more.len(), 3);
    }
    let read = bytes_read.get();
    assert!(
        read < total,
        "early stop should leave unread input: read {read} of {total}"
    );
}

#[test]
fn malformed_input_returns_parse_error() {
    let err = Parser::for_syntax(Syntax::Turtle)
        .parse_str("<https://example.com/s> <https://example.com/p> .")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap_err();
    assert!(matches!(err, Error::Parse(_)));
}

#[test]
fn progressive_load_leaves_partial_data_on_failure() {
    let input = "\
<https://example.com/a> <https://example.com/p> \"ok\" .
<https://example.com/b> <https://example.com/p> .
";
    let model = Model::new().unwrap();
    let err = Parser::for_syntax(Syntax::NTriples)
        .load_into(&model, input.as_bytes())
        .unwrap_err();
    assert!(matches!(err, Error::Parse(_)));
    assert!(
        err.to_string().contains("partial load"),
        "error should document partial load: {err}"
    );
    assert_eq!(model.len().unwrap(), 1);
}

#[test]
fn collecting_load_is_all_or_nothing() {
    let input = "\
<https://example.com/a> <https://example.com/p> \"ok\" .
<https://example.com/b> <https://example.com/p> .
";
    let model = Model::new().unwrap();
    let err = Parser::for_syntax(Syntax::NTriples)
        .load_collecting(&model, input.as_bytes())
        .unwrap_err();
    assert!(matches!(err, Error::Parse(_)));
    assert!(model.is_empty().unwrap());
}

#[test]
fn named_graph_target_and_dataset_trig() {
    let turtle = "<https://example.com/a> <https://example.com/p> \"x\" .";
    let graph = GraphName::NamedNode(NamedNode::new("https://example.com/g").unwrap());
    let model = Model::new().unwrap();
    Parser::for_syntax(Syntax::Turtle)
        .graph_target(GraphTarget::Named(graph.clone()))
        .load_collecting(&model, turtle.as_bytes())
        .unwrap();
    let statement = Triple::new(
        NamedNode::new("https://example.com/a").unwrap(),
        NamedNode::new("https://example.com/p").unwrap(),
        Literal::new_simple_literal("x"),
    );
    assert!(!model.contains(statement.as_ref()).unwrap());
    assert!(
        model
            .contains_in_graph(statement.as_ref(), (&graph).into())
            .unwrap()
    );

    let trig = "{ <https://example.com/a> <https://example.com/p> \"d\" . }
<https://example.com/g> { <https://example.com/a> <https://example.com/p> \"n\" . }
";
    let dataset = Model::new().unwrap();
    Parser::for_syntax(Syntax::TriG)
        .graph_target(GraphTarget::Dataset)
        .load_collecting(&dataset, trig.as_bytes())
        .unwrap();
    assert_eq!(dataset.len().unwrap(), 2);
    assert!(
        dataset
            .contains(
                Triple::new(
                    NamedNode::new("https://example.com/a").unwrap(),
                    NamedNode::new("https://example.com/p").unwrap(),
                    Literal::new_simple_literal("d"),
                )
                .as_ref()
            )
            .unwrap()
    );
}

#[test]
fn base_iri_resolves_relative_terms() {
    let turtle = "<alice> <name> \"Alice\" .";
    let quads = Parser::for_syntax(Syntax::Turtle)
        .base_iri("https://example.com/")
        .unwrap()
        .parse_str(turtle)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(quads.len(), 1);
    assert_eq!(
        quads[0].subject,
        terms::named_node("https://example.com/alice")
            .unwrap()
            .into()
    );
}

#[test]
fn language_and_datatype_literals_round_trip() {
    let turtle = r#"
@prefix ex: <https://example.com/> .
ex:s ex:p "chat"@en .
ex:s ex:q "1"^^<http://www.w3.org/2001/XMLSchema#integer> .
"#;
    let model = Model::new().unwrap();
    Parser::for_syntax(Syntax::Turtle)
        .load_collecting(&model, turtle.as_bytes())
        .unwrap();
    assert_eq!(model.len().unwrap(), 2);
    let out = Serializer::for_syntax(Syntax::Turtle)
        .with_prefix("ex", "https://example.com/")
        .unwrap()
        .serialize_model_to_string(&model)
        .unwrap();
    let again = Model::new().unwrap();
    Parser::for_syntax(Syntax::Turtle)
        .load_collecting(&again, out.as_bytes())
        .unwrap();
    assert_eq!(again.len().unwrap(), 2);
}

#[test]
fn dataset_serializer_preserves_named_graphs() {
    let model = Model::new().unwrap();
    let triple = alice_triple();
    model.add(triple.clone()).unwrap();
    model
        .add_to_graph(
            triple,
            GraphName::NamedNode(NamedNode::new("https://example.com/people").unwrap()),
        )
        .unwrap();
    let nq = Serializer::for_syntax(Syntax::NQuads)
        .serialize_model_to_string(&model)
        .unwrap();
    let reloaded = Model::new().unwrap();
    Parser::for_syntax(Syntax::NQuads)
        .graph_target(GraphTarget::Dataset)
        .load_collecting(&reloaded, nq.as_bytes())
        .unwrap();
    assert_eq!(reloaded.len().unwrap(), 2);
}

#[test]
fn graph_only_serializer_rejects_named_graphs() {
    let model = Model::new().unwrap();
    model
        .add_to_graph(
            alice_triple(),
            GraphName::NamedNode(NamedNode::new("https://example.com/g").unwrap()),
        )
        .unwrap();
    let err = Serializer::for_syntax(Syntax::Turtle)
        .serialize_model_to_string(&model)
        .unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)));
}

#[test]
fn blank_nodes_do_not_collide_across_parses() {
    let input = "_:b <https://example.com/p> \"x\" .";
    let model = Model::new().unwrap();
    Parser::for_syntax(Syntax::Turtle)
        .load_collecting(&model, input.as_bytes())
        .unwrap();
    Parser::for_syntax(Syntax::Turtle)
        .load_collecting(&model, input.as_bytes())
        .unwrap();
    assert_eq!(model.len().unwrap(), 2);
}

#[test]
fn file_round_trip_and_extension_detection() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.ttl");
    let model = Model::new().unwrap();
    model.add(alice_triple()).unwrap();
    Serializer::for_syntax(Syntax::Turtle)
        .serialize_model_to_path(&model, &path)
        .unwrap();
    let (syntax, stream) = Parser::parse_path_with_extension(&path).unwrap();
    assert_eq!(syntax, Syntax::Turtle);
    let quads = stream.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(quads.len(), 1);
}

#[test]
fn unicode_literals_are_preserved() {
    let turtle = r#"<https://example.com/s> <https://example.com/p> "日本語🍕" ."#;
    let quads = Parser::for_syntax(Syntax::Turtle)
        .parse_str(turtle)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(quads.len(), 1);
    match &quads[0].object {
        oxiland::terms::Term::Literal(lit) => assert_eq!(lit.value(), "日本語🍕"),
        other => panic!("expected literal, got {other:?}"),
    }
}

#[test]
fn failing_reader_preserves_io_category() {
    struct Boom;
    impl Read for Boom {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("boom"))
        }
    }
    let err = Parser::for_syntax(Syntax::NTriples)
        .parse_reader(Boom)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap_err();
    assert!(matches!(err, Error::Io(_)));
}

#[test]
fn dataset_graph_target_rejected_for_turtle() {
    let result = Parser::for_syntax(Syntax::Turtle)
        .graph_target(GraphTarget::Dataset)
        .parse_str("");
    assert!(matches!(result, Err(Error::Unsupported(_))));
}

#[test]
fn dataset_default_graph_target_rejects_named_quads() {
    let nq = "<https://example.com/a> <https://example.com/p> \"x\" <https://example.com/g> .\n";
    let err = Parser::for_syntax(Syntax::NQuads)
        .graph_target(GraphTarget::DefaultGraph)
        .parse_str(nq)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap_err();
    assert!(matches!(err, Error::Parse(_)), "got {err}");
}

#[test]
fn nquads_named_target_rejects_foreign_named_graphs() {
    let nq =
        "<https://example.com/a> <https://example.com/p> \"x\" <https://example.com/other> .\n";
    let target = GraphName::NamedNode(NamedNode::new("https://example.com/g").unwrap());
    let err = Parser::for_syntax(Syntax::NQuads)
        .graph_target(GraphTarget::Named(target))
        .parse_str(nq)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap_err();
    assert!(matches!(err, Error::Parse(_)), "got {err}");
}

#[test]
fn parse_error_display_does_not_duplicate_location() {
    let err = Parser::for_syntax(Syntax::Turtle)
        .parse_str("<https://example.com/s> <https://example.com/p> .")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap_err();
    let text = err.to_string();
    let occurrences = text.matches("line ").count();
    assert!(
        occurrences <= 1,
        "location should appear at most once: {text}"
    );
    if let Error::Parse(parse) = err {
        assert!(parse.location.is_some());
        assert!(!parse.message.contains(" at line "));
    } else {
        panic!("expected Parse error");
    }
}

#[test]
fn prefix_rejected_for_ntriples() {
    let err = Serializer::for_syntax(Syntax::NTriples)
        .with_prefix("ex", "https://example.com/")
        .unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)));
}

#[test]
fn fjall_progressive_load_persists_partial_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store");
    let input = "\
<https://example.com/a> <https://example.com/p> \"ok\" .
<https://example.com/b> <https://example.com/p> .
";
    {
        let model = Model::open(&path).unwrap();
        let err = Parser::for_syntax(Syntax::NTriples)
            .load_into(&model, input.as_bytes())
            .unwrap_err();
        assert!(matches!(err, Error::Parse(_)));
        assert_eq!(model.len().unwrap(), 1);
    }
    let reopened = Model::open(&path).unwrap();
    assert_eq!(reopened.len().unwrap(), 1);
    assert!(
        reopened
            .contains(
                Triple::new(
                    NamedNode::new("https://example.com/a").unwrap(),
                    NamedNode::new("https://example.com/p").unwrap(),
                    Literal::new_simple_literal("ok"),
                )
                .as_ref()
            )
            .unwrap()
    );
}

#[test]
fn large_stream_parse_does_not_require_full_collect_api() {
    // Construct a multi-megabyte N-Triples document and ensure we can iterate
    // without an intermediate Vec of the whole file contents at the facade.
    let mut input = Vec::with_capacity(2 * 1024 * 1024);
    for i in 0..50_000 {
        let line = format!("<https://example.com/s/{i}> <https://example.com/p> \"value-{i}\" .\n");
        input.extend_from_slice(line.as_bytes());
    }
    let mut count = 0usize;
    for item in Parser::for_syntax(Syntax::NTriples)
        .parse_reader(Cursor::new(input))
        .unwrap()
    {
        let _quad: Quad = item.unwrap();
        count += 1;
        if count == 100 {
            break;
        }
    }
    assert_eq!(count, 100);
}

#[test]
fn parse_path_with_extension_accepts_nquads_named_graphs() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.nq");
    std::fs::write(
        &path,
        "<https://example.com/a> <https://example.com/p> \"x\" <https://example.com/g> .\n",
    )
    .unwrap();
    let (syntax, stream) = Parser::parse_path_with_extension(&path).unwrap();
    assert_eq!(syntax, Syntax::NQuads);
    let quads = stream.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(quads.len(), 1);
    assert_eq!(
        quads[0].graph_name,
        GraphName::NamedNode(NamedNode::new("https://example.com/g").unwrap())
    );
}

#[test]
fn named_graph_target_accepts_matching_named_quads() {
    let nq = "<https://example.com/a> <https://example.com/p> \"x\" <https://example.com/g> .\n";
    let target = GraphName::NamedNode(NamedNode::new("https://example.com/g").unwrap());
    let quads = Parser::for_syntax(Syntax::NQuads)
        .graph_target(GraphTarget::Named(target.clone()))
        .parse_str(nq)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(quads.len(), 1);
    assert_eq!(quads[0].graph_name, target);
}

#[test]
fn progressive_load_annotates_partial_data_on_io_failure() {
    struct BoomAfterFirstTriple {
        offset: usize,
    }
    impl Read for BoomAfterFirstTriple {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            const CHUNK: &[u8] = b"<https://example.com/a> <https://example.com/p> \"ok\" .\n";
            if self.offset < CHUNK.len() {
                let n = (CHUNK.len() - self.offset).min(buf.len());
                buf[..n].copy_from_slice(&CHUNK[self.offset..self.offset + n]);
                self.offset += n;
                return Ok(n);
            }
            Err(std::io::Error::other("boom after first triple"))
        }
    }

    let model = Model::new().unwrap();
    let err = Parser::for_syntax(Syntax::NTriples)
        .load_into(&model, BoomAfterFirstTriple { offset: 0 })
        .unwrap_err();
    assert!(matches!(err, Error::Io(_)));
    assert!(
        err.to_string().contains("partial load"),
        "error should document partial load: {err}"
    );
    assert_eq!(model.len().unwrap(), 1);
}

#[test]
fn parse_str_and_slice_strip_utf8_bom() {
    let turtle = "\u{feff}<https://example.com/s> <https://example.com/p> \"bom\" .\n";
    let quads = Parser::for_syntax(Syntax::Turtle)
        .parse_str(turtle)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(quads.len(), 1);

    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(b"<https://example.com/s> <https://example.com/p> \"bom\" .\n");
    let quads = Parser::for_syntax(Syntax::NTriples)
        .parse_slice(&bytes)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(quads.len(), 1);
}

#[test]
fn load_transactional_strips_utf8_bom() {
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(b"<https://example.com/s> <https://example.com/p> \"bom-tx\" .\n");
    let model = Model::new().unwrap();
    assert_eq!(
        Parser::for_syntax(Syntax::Turtle)
            .load_transactional(&model, &bytes[..])
            .unwrap(),
        1
    );
    assert_eq!(model.len().unwrap(), 1);
}
