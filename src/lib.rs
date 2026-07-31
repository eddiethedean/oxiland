//! A safe Rust successor to the Redland `librdf` API, powered by Oxigraph.
//!
//! Oxiland follows Redland's object model without copying its manual-memory
//! ownership rules into safe Rust. Oxigraph RDF term types are re-exported so
//! callers can move data into and out of the underlying RDF ecosystem without
//! adapters (see ADR-004).
//!
//! # Quick start
//!
//! ```
//! use oxiland::terms::{self, Literal, Triple};
//! use oxiland::{Model, Query, QueryResults};
//!
//! # fn main() -> oxiland::Result<()> {
//! let model = Model::new()?;
//! model.add(Triple::new(
//!     terms::named_node("https://example.com/alice")?,
//!     terms::named_node("https://example.com/name")?,
//!     Literal::new_simple_literal("Alice"),
//! ))?;
//!
//! assert!(matches!(
//!     Query::new("ASK { ?s ?p ?o }").execute(&model)?,
//!     QueryResults::Boolean(true)
//! ));
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
pub mod io;
mod model;
mod persist;
mod query;
pub mod storage;
pub mod utility;
mod world;

pub use error::{Error, ParseError, Result};
pub use model::{Model, ModelTransaction, StatementMatches, StatementPattern};
pub use query::{
    Query, QueryResults, ResultsFormat, Update, serialize_graph_results_to_writer,
    serialize_query_results_to_string, serialize_query_results_to_writer,
};
pub use storage::{OpenOptions, StorageBackend, StorageCapabilities};
pub use world::{FeatureValue, LogFacility, LogLevel, LogRecord, World};

/// RDF term, triple, quad, and graph-name types used by Oxiland.
///
/// These are direct re-exports of Oxigraph types (ADR-004). Prefer the helpers
/// below when you want Oxiland [`Error`] categories instead of Oxigraph errors.
pub mod terms {
    pub use oxigraph::model::{
        BlankNode, GraphName, GraphNameRef, Literal, NamedNode, NamedNodeRef, NamedOrBlankNode,
        NamedOrBlankNodeRef, Quad, QuadRef, Term, TermRef, Triple, TripleRef, Variable,
    };

    use crate::{Error, Result};

    /// Creates a [`NamedNode`], mapping IRI failures to [`Error::InvalidRdf`].
    pub fn named_node(iri: impl AsRef<str>) -> Result<NamedNode> {
        NamedNode::new(iri.as_ref()).map_err(|error| Error::InvalidRdf(error.to_string()))
    }

    /// Creates a [`BlankNode`] from an optional identifier.
    ///
    /// When `id` is `None`, Oxigraph allocates a fresh blank node. Invalid
    /// identifiers map to [`Error::InvalidRdf`].
    pub fn blank_node(id: Option<&str>) -> Result<BlankNode> {
        match id {
            Some(id) => BlankNode::new(id).map_err(|error| Error::InvalidRdf(error.to_string())),
            None => Ok(BlankNode::default()),
        }
    }
}

/// Oxigraph SPARQL primitives for advanced use cases.
///
/// Prefer [`Query`], [`Update`], and [`ResultsFormat`] for Redland-shaped
/// workflows. This module is an escape hatch, not the compatibility surface.
pub mod sparql {
    pub use oxigraph::sparql::results::{QueryResultsFormat, QueryResultsSerializer};
    pub use oxigraph::sparql::{
        CancellationToken, QueryResults, QuerySolution, QuerySolutionIter, QueryTripleIter,
        SparqlEvaluator,
    };
}
