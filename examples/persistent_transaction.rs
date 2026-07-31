//! Persistent model with a transaction (Oxiland 0.4).

use oxiland::Model;
use oxiland::terms::{self, Literal, Triple};

fn main() -> oxiland::Result<()> {
    let dir = tempfile::tempdir()?;
    let model = Model::open(dir.path())?;

    model.transaction(|tx| {
        tx.add(Triple::new(
            terms::named_node("https://example.com/alice")?,
            terms::named_node("https://example.com/name")?,
            Literal::new_simple_literal("Alice"),
        ))?;
        Ok(())
    })?;

    println!("quads after commit: {}", model.len()?);
    model.sync()?;
    Ok(())
}
