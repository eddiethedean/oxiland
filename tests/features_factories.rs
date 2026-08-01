//! 0.10 feature, factory, storage-facade, and world-bridge tests.

use oxiland::io::{Parser, Serializer, Syntax};
use oxiland::terms::{self, Literal, Triple};
use oxiland::{FeatureValue, Model, StatementPattern, World, factory};

#[test]
fn factory_registration_is_idempotent_for_builtins() {
    factory::register_parser_factory("turtle").unwrap();
    factory::register_parser_factory("turtle").unwrap();
    factory::register_serializer_factory("ntriples").unwrap();
    factory::register_storage_factory("memory").unwrap();
    factory::register_storage_factory("fjall").unwrap();
    factory::register_query_factory("sparql").unwrap();
    assert!(factory::parser_factory_registered("turtle"));
    assert!(factory::storage_factory_registered("memory"));
    assert!(factory::query_factory_registered("sparql"));
    assert!(factory::register_query_factory("unknown-engine").is_err());
}

#[test]
fn world_bridges_and_factories_round_trip() {
    let world = World::new();
    world.set_raptor(Some(0xABCD));
    world.set_raptor_init_handler(Some(0x1111));
    world.set_rasqal(Some(0x2222));
    assert_eq!(world.raptor(), Some(0xABCD));
    assert_eq!(world.raptor_init_handler(), Some(0x1111));
    assert_eq!(world.rasqal(), Some(0x2222));
    world.register_parser_factory("trig").unwrap();
}

#[test]
fn model_parser_serializer_features() {
    let model = Model::new().unwrap();
    model.set_feature("http://example.com/m", FeatureValue::Boolean(true));
    assert_eq!(
        model.feature("http://example.com/m"),
        Some(FeatureValue::Boolean(true))
    );

    let parser = Parser::for_syntax(Syntax::Turtle);
    parser.set_feature("http://example.com/p", FeatureValue::Integer(3));
    assert_eq!(
        parser.feature("http://example.com/p"),
        Some(FeatureValue::Integer(3))
    );

    let serializer = Serializer::for_syntax(Syntax::NTriples);
    serializer.set_feature("http://example.com/s", FeatureValue::String("x".into()));
    assert_eq!(
        serializer.feature("http://example.com/s"),
        Some(FeatureValue::String("x".into()))
    );
}

#[test]
fn storage_facade_mirrors_model_ops() {
    let model = Model::new().unwrap();
    let statement = Triple::new(
        terms::named_node("https://example.com/s").unwrap(),
        terms::named_node("https://example.com/p").unwrap(),
        Literal::new_simple_literal("o"),
    );
    let storage = model.as_storage();
    assert!(storage.add_statement(statement.clone()).unwrap());
    assert!(storage.contains_statement(statement.as_ref()).unwrap());
    assert_eq!(storage.size().unwrap(), 1);
    storage.set_feature("http://example.com/sf", FeatureValue::Boolean(false));
    assert_eq!(
        storage.feature("http://example.com/sf"),
        Some(FeatureValue::Boolean(false))
    );
    let found = storage
        .find_statements(StatementPattern::default())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(found.len(), 1);
    storage.close().unwrap();
}
