use std::path::PathBuf;

/// Errors produced by Oxiland.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An RDF term or IRI was invalid.
    #[error("invalid RDF value: {0}")]
    InvalidRdf(String),
    /// A SPARQL query could not be parsed.
    #[error("SPARQL parse error: {0}")]
    SparqlParse(String),
    /// A SPARQL query could not be evaluated.
    #[error("SPARQL evaluation error: {0}")]
    SparqlEvaluation(String),
    /// The storage backend failed.
    #[error("storage error: {0}")]
    Storage(String),
    /// A requested Redland-compatible feature is unsupported.
    #[error("unsupported feature: {0}")]
    Unsupported(String),
    /// A filesystem-backed store could not be opened.
    #[error("could not open store at {}: {message}", path.display())]
    OpenStore {
        /// Store path.
        path: PathBuf,
        /// Backend error.
        message: String,
    },
}

/// Result type used throughout Oxiland.
pub type Result<T> = std::result::Result<T, Error>;
