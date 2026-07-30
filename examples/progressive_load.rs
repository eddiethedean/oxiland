//! Demonstrate progressive vs collecting parser loads (ADR-007).

use oxiland::Model;
use oxiland::io::{Parser, Syntax};

fn main() -> oxiland::Result<()> {
    let input = "\
<https://example.com/a> <https://example.com/p> \"ok\" .
<https://example.com/b> <https://example.com/p> .
";

    let progressive = Model::new()?;
    match Parser::for_syntax(Syntax::NTriples).load_into(&progressive, input.as_bytes()) {
        Ok(_) => panic!("expected parse failure"),
        Err(error) => {
            println!("progressive error: {error}");
            println!("statements left in model: {}", progressive.len()?);
        }
    }

    let collecting = Model::new()?;
    match Parser::for_syntax(Syntax::NTriples).load_collecting(&collecting, input.as_bytes()) {
        Ok(_) => panic!("expected parse failure"),
        Err(error) => {
            println!("collecting error: {error}");
            println!("statements left in model: {}", collecting.len()?);
        }
    }

    Ok(())
}
