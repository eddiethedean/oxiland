//! Obligation-linked safe workflow smoke for milestone 0.11.
//!
//! These tests exercise the safe Rust facades that back the 0.11 differential
//! fixtures. Full verification remains gated on raw two-sided harness results.

use std::io::Cursor;

use oxiland::io::{Parser, Serializer, Syntax};
use oxiland::terms::{self, Literal, Triple};
use oxiland::{Model, Query, QueryResults, StatementPattern};

#[test]
fn obl_world_and_model_lifecycle() -> oxiland::Result<()> {
    let model = Model::new()?;
    assert!(model.is_empty()?);
    Ok(())
}

#[test]
fn obl_parse_turtle_ask_select() -> oxiland::Result<()> {
    let model = Model::new()?;
    let turtle = r#"<http://example.org/alice> <http://schema.org/name> "Alice" ."#;
    let n = Parser::for_syntax(Syntax::Turtle).load_into(&model, Cursor::new(turtle.as_bytes()))?;
    assert_eq!(n, 1);
    assert!(matches!(
        Query::new("ASK { ?s ?p ?o }").execute(&model)?,
        QueryResults::Boolean(true)
    ));
    let results = Query::new("SELECT ?s WHERE { ?s ?p ?o }").execute(&model)?;
    match results {
        QueryResults::Solutions(iter) => {
            assert_eq!(iter.count(), 1);
        }
        other => panic!("expected solutions, got {other:?}"),
    }
    Ok(())
}

#[test]
fn obl_statement_model_add() -> oxiland::Result<()> {
    let model = Model::new()?;
    assert!(model.add(Triple::new(
        terms::named_node("http://example.org/alice")?,
        terms::named_node("http://schema.org/name")?,
        Literal::new_simple_literal("Alice"),
    ))?);
    assert_eq!(model.len()?, 1);
    Ok(())
}

#[test]
fn obl_serialize_ntriples() -> oxiland::Result<()> {
    let model = Model::new()?;
    model.add(Triple::new(
        terms::named_node("http://example.org/s")?,
        terms::named_node("http://example.org/p")?,
        terms::named_node("http://example.org/o")?,
    ))?;
    let text = Serializer::for_syntax(Syntax::NTriples).serialize_model_to_string(&model)?;
    assert!(text.contains("<http://example.org/s>"));
    Ok(())
}

#[test]
fn obl_stream_find_count() -> oxiland::Result<()> {
    let model = Model::new()?;
    model.add(Triple::new(
        terms::named_node("http://example.org/a")?,
        terms::named_node("http://example.org/p")?,
        terms::named_node("http://example.org/b")?,
    ))?;
    assert_eq!(model.find(StatementPattern::default()).count(), 1);
    Ok(())
}
