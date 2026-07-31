use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::io::SourceLocation;

/// Errors produced by Oxiland.
///
/// The variant set is closed for 1.0 intent (ADR-020). New categories require an
/// ADR. Unsupported Redland factories and options surface as [`Error::Unsupported`]
/// rather than silent success.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An RDF term or IRI was invalid.
    #[error("invalid RDF value: {0}")]
    InvalidRdf(String),
    /// An RDF document could not be parsed.
    #[error("{0}")]
    Parse(#[from] ParseError),
    /// An RDF document could not be serialized.
    #[error("RDF serialize error: {0}")]
    Serialize(String),
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
    /// A filesystem or byte-stream I/O failure.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// A filesystem-backed store could not be opened.
    #[error("could not open store at {}: {message}", path.display())]
    OpenStore {
        /// Store path.
        path: PathBuf,
        /// Backend error.
        message: String,
    },
}

/// Structured RDF parse failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    /// Human-readable parse diagnostic.
    pub message: String,
    /// Source location when the underlying engine provides one.
    pub location: Option<SourceLocation>,
}

impl ParseError {
    pub(crate) fn new(message: impl Into<String>, location: Option<SourceLocation>) -> Self {
        Self {
            message: message.into(),
            location,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RDF parse error")?;
        if let Some(location) = &self.location {
            write!(f, "{location}")?;
        }
        write!(f, ": {}", self.message)
    }
}

impl std::error::Error for ParseError {}

impl Error {
    pub(crate) fn parse(message: impl Into<String>, location: Option<SourceLocation>) -> Self {
        Self::Parse(ParseError::new(message, location))
    }
}

/// Result type used throughout Oxiland.
pub type Result<T> = std::result::Result<T, Error>;
