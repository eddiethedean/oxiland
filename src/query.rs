use oxigraph::sparql::SparqlEvaluator;

use crate::{Error, Model, Result};

/// Results returned by a SPARQL query.
pub type QueryResults<'a> = oxigraph::sparql::QueryResults<'a>;

/// A SPARQL query that is parsed when executed.
///
/// # Examples
///
/// ```
/// use oxiland::terms::{self, Literal, Triple};
/// use oxiland::{Model, Query, QueryResults};
///
/// # fn main() -> oxiland::Result<()> {
/// let model = Model::new()?;
/// model.add(Triple::new(
///     terms::named_node("https://example.com/alice")?,
///     terms::named_node("https://example.com/name")?,
///     Literal::new_simple_literal("Alice"),
/// ))?;
///
/// let results = Query::new("ASK { ?s ?p ?o }").execute(&model)?;
/// assert!(matches!(results, QueryResults::Boolean(true)));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Query {
    text: String,
}

impl Query {
    /// Creates a SPARQL query from its text.
    ///
    /// The query is not parsed until [`Query::execute`].
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Returns the query text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Parses and executes the query against a model.
    ///
    /// Parse failures return [`Error::SparqlParse`]. Evaluation failures return
    /// [`Error::SparqlEvaluation`].
    pub fn execute<'a>(&self, model: &'a Model) -> Result<QueryResults<'a>> {
        SparqlEvaluator::new()
            .parse_query(&self.text)
            .map_err(|error| Error::SparqlParse(error.to_string()))?
            .on_store(model.store())
            .execute()
            .map_err(|error| Error::SparqlEvaluation(error.to_string()))
    }
}
