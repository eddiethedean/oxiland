//! A safe Rust successor to the Redland `librdf` API, powered by Oxigraph.
//!
//! Oxiland follows Redland's object model without copying its manual-memory
//! ownership rules into safe Rust. Oxigraph types are re-exported so callers
//! can move data into and out of the underlying RDF ecosystem without adapters.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod model;
mod query;
mod world;

pub use error::{Error, Result};
pub use model::{Model, StatementPattern};
pub use query::{Query, QueryResults};
pub use world::{FeatureValue, World};

/// RDF term, triple, quad, and graph-name types used by Oxiland.
pub mod terms {
    pub use oxigraph::model::{
        BlankNode, GraphName, GraphNameRef, Literal, NamedNode, NamedNodeRef, NamedOrBlankNode,
        NamedOrBlankNodeRef, Quad, QuadRef, Term, TermRef, Triple, TripleRef, Variable,
    };
}

/// RDF parsing and serialization primitives.
pub mod io {
    pub use oxigraph::io::{RdfFormat, RdfParser, RdfSerializer};
}

/// Oxigraph SPARQL primitives for advanced use cases.
pub mod sparql {
    pub use oxigraph::sparql::{QueryResults, QuerySolution, QuerySolutionIter, SparqlEvaluator};
}
