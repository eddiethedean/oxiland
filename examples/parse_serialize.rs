//! Parse Turtle and serialize N-Triples with the Oxiland I/O facade.

use oxiland::Model;
use oxiland::io::{Parser, Serializer, Syntax};

fn main() -> oxiland::Result<()> {
    let model = Model::new()?;
    Parser::for_syntax(Syntax::Turtle)
        .base_iri("https://example.com/")?
        .load_collecting(&model, b"<alice> <name> \"Alice\" .".as_slice())?;

    let ntriples = Serializer::for_syntax(Syntax::NTriples).serialize_model_to_string(&model)?;
    println!("{ntriples}");
    Ok(())
}
