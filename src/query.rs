use oxigraph::sparql::SparqlEvaluator;

use crate::{Error, Model, Result};

/// Results returned by a SPARQL query.
pub type QueryResults<'a> = oxigraph::sparql::QueryResults<'a>;

/// A parsed-on-execution SPARQL query.
#[derive(Clone, Debug)]
pub struct Query {
    text: String,
}

impl Query {
    /// Creates a SPARQL query.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Returns the query text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Executes the query against a model.
    pub fn execute<'a>(&self, model: &'a Model) -> Result<QueryResults<'a>> {
        SparqlEvaluator::new()
            .parse_query(&self.text)
            .map_err(|error| Error::Sparql(error.to_string()))?
            .on_store(model.store())
            .execute()
            .map_err(|error| Error::Sparql(error.to_string()))
    }
}
