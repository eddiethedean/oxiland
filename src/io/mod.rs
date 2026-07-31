//! Redland-shaped RDF parsing and serialization.
//!
//! Oxiland provides safe [`Parser`] and [`Serializer`] facades with closed
//! [`Syntax`] discovery (ADR-008). Streaming parse output and progressive
//! model loading follow ADR-007.
//!
//! Oxigraph primitives remain available under [`primitives`] for advanced use;
//! they are not evidence for Redland-shaped facade completeness.

mod format;
mod location;
mod parser;
mod serializer;

pub use format::Syntax;
pub use location::SourceLocation;
pub use parser::{GraphTarget, Parser, QuadStream, SliceStream};
pub use serializer::Serializer;

/// Direct Oxigraph I/O primitives.
///
/// Prefer [`Parser`] and [`Serializer`] for Redland-oriented workflows.
pub mod primitives {
    pub use oxigraph::io::{
        RdfFormat, RdfParseError, RdfParser, RdfSerializer, RdfSyntaxError, TextPosition,
    };
}
