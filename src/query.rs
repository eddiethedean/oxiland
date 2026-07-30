//! Redland-shaped SPARQL query, update, and results (0.3).
//!
//! See ADR-009–ADR-012 and `docs/design/0.3-query-api.md`.

use std::io::Write;

use oxigraph::model::{GraphName, NamedOrBlankNode};
use oxigraph::sparql::results::{QueryResultsFormat, QueryResultsSerializer};
use oxigraph::sparql::{CancellationToken, PreparedSparqlQuery, SparqlEvaluator};
use spargebra::algebra::GraphPattern;
use spargebra::{Query as SparqlAlgebraQuery, SparqlParser};

use crate::{Error, Model, Result};

/// Results returned by a SPARQL query (Oxigraph enum; ADR-010).
pub type QueryResults<'a> = oxigraph::sparql::QueryResults<'a>;

/// Dataset configuration applied at execute time (ADR-009).
#[derive(Clone, Debug, Default)]
struct DatasetConfig {
    default_graphs: Option<Vec<GraphName>>,
    default_as_union: bool,
    named_graphs: Option<Vec<NamedOrBlankNode>>,
}

/// A SPARQL query configured before execution (ADR-009).
///
/// # Examples
///
/// ASK:
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
///
/// SELECT with limit:
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
/// let results = Query::new(
///     "SELECT ?name WHERE { <https://example.com/alice> <https://example.com/name> ?name }",
/// )
/// .limit(1)?
/// .execute(&model)?;
///
/// if let QueryResults::Solutions(mut solutions) = results {
///     let solution = solutions
///         .next()
///         .expect("one row")
///         .map_err(|error| oxiland::Error::SparqlEvaluation(error.to_string()))?;
///     assert!(solution.get("name").is_some());
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Query {
    text: String,
    base_iri: Option<String>,
    prefixes: Vec<(String, String)>,
    limit: Option<usize>,
    offset: usize,
    dataset: DatasetConfig,
    cancellation: Option<CancellationToken>,
}

impl Query {
    /// Creates a SPARQL query from its text.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            base_iri: None,
            prefixes: Vec::new(),
            limit: None,
            offset: 0,
            dataset: DatasetConfig::default(),
            cancellation: None,
        }
    }

    /// Returns the query text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Sets the base IRI used while parsing the query.
    pub fn base_iri(mut self, base_iri: impl Into<String>) -> Result<Self> {
        let base_iri = base_iri.into();
        let _ = SparqlParser::new()
            .with_base_iri(&base_iri)
            .map_err(|error| Error::InvalidRdf(error.to_string()))?;
        self.base_iri = Some(base_iri);
        Ok(self)
    }

    /// Adds a default IRI prefix used while parsing the query.
    pub fn prefix(
        mut self,
        prefix_name: impl Into<String>,
        prefix_iri: impl Into<String>,
    ) -> Result<Self> {
        let prefix_name = prefix_name.into();
        let prefix_iri = prefix_iri.into();
        let _ = SparqlParser::new()
            .with_prefix(&prefix_name, &prefix_iri)
            .map_err(|error| Error::InvalidRdf(error.to_string()))?;
        self.prefixes.push((prefix_name, prefix_iri));
        Ok(self)
    }

    /// Sets an API-level result limit (algebra `Slice`; ADR-009).
    pub fn limit(mut self, limit: usize) -> Result<Self> {
        self.limit = Some(limit);
        Ok(self)
    }

    /// Sets an API-level result offset (algebra `Slice`; ADR-009).
    pub fn offset(mut self, offset: usize) -> Result<Self> {
        self.offset = offset;
        Ok(self)
    }

    /// Restricts the query default graph to the given graph names.
    #[must_use]
    pub fn default_graph(mut self, graphs: impl IntoIterator<Item = GraphName>) -> Self {
        self.dataset.default_graphs = Some(graphs.into_iter().collect());
        self.dataset.default_as_union = false;
        self
    }

    /// Treats the union of all named graphs as the default graph.
    #[must_use]
    pub fn default_graph_as_union(mut self) -> Self {
        self.dataset.default_as_union = true;
        self.dataset.default_graphs = None;
        self
    }

    /// Restricts available named graphs for the query dataset.
    #[must_use]
    pub fn available_named_graphs(
        mut self,
        graphs: impl IntoIterator<Item = NamedOrBlankNode>,
    ) -> Self {
        self.dataset.named_graphs = Some(graphs.into_iter().collect());
        self
    }

    /// Attaches a cooperative cancellation token (ADR-012).
    ///
    /// Wall-clock timeouts are caller-driven: cancel the token from another
    /// thread when a deadline expires.
    #[must_use]
    pub fn cancellation_token(mut self, token: CancellationToken) -> Self {
        self.cancellation = Some(token);
        self
    }

    /// Parses and executes the query against a model.
    ///
    /// Parse failures return [`Error::SparqlParse`]. Evaluation failures return
    /// [`Error::SparqlEvaluation`].
    pub fn execute<'a>(&self, model: &'a Model) -> Result<QueryResults<'a>> {
        let prepared = self.prepare()?;
        prepared
            .on_store(model.store())
            .execute()
            .map_err(|error| Error::SparqlEvaluation(error.to_string()))
    }

    fn prepare(&self) -> Result<PreparedSparqlQuery> {
        let mut parser = SparqlParser::new();
        if let Some(base_iri) = &self.base_iri {
            parser = parser
                .with_base_iri(base_iri)
                .map_err(|error| Error::SparqlParse(error.to_string()))?;
        }
        for (name, iri) in &self.prefixes {
            parser = parser
                .with_prefix(name, iri)
                .map_err(|error| Error::SparqlParse(error.to_string()))?;
        }
        let mut algebra = parser
            .parse_query(&self.text)
            .map_err(|error| Error::SparqlParse(error.to_string()))?;
        algebra = apply_slice(algebra, self.offset, self.limit)?;

        let mut evaluator = SparqlEvaluator::new();
        if let Some(token) = &self.cancellation {
            evaluator = evaluator.with_cancellation_token(token.clone());
        }
        let mut prepared = evaluator.for_query(algebra);
        apply_dataset(prepared.dataset_mut(), &self.dataset);
        Ok(prepared)
    }
}

/// A SPARQL Update operation configured before execution (ADR-009).
#[derive(Clone)]
pub struct Update {
    text: String,
    base_iri: Option<String>,
    prefixes: Vec<(String, String)>,
    dataset: DatasetConfig,
    cancellation: Option<CancellationToken>,
}

impl Update {
    /// Creates a SPARQL Update from its text.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            base_iri: None,
            prefixes: Vec::new(),
            dataset: DatasetConfig::default(),
            cancellation: None,
        }
    }

    /// Returns the update text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Sets the base IRI used while parsing the update.
    pub fn base_iri(mut self, base_iri: impl Into<String>) -> Result<Self> {
        let base_iri = base_iri.into();
        let _ = SparqlParser::new()
            .with_base_iri(&base_iri)
            .map_err(|error| Error::InvalidRdf(error.to_string()))?;
        self.base_iri = Some(base_iri);
        Ok(self)
    }

    /// Adds a default IRI prefix used while parsing the update.
    pub fn prefix(
        mut self,
        prefix_name: impl Into<String>,
        prefix_iri: impl Into<String>,
    ) -> Result<Self> {
        let prefix_name = prefix_name.into();
        let prefix_iri = prefix_iri.into();
        let _ = SparqlParser::new()
            .with_prefix(&prefix_name, &prefix_iri)
            .map_err(|error| Error::InvalidRdf(error.to_string()))?;
        self.prefixes.push((prefix_name, prefix_iri));
        Ok(self)
    }

    /// Restricts USING-style dataset default graphs when applicable.
    #[must_use]
    pub fn default_graph(mut self, graphs: impl IntoIterator<Item = GraphName>) -> Self {
        self.dataset.default_graphs = Some(graphs.into_iter().collect());
        self.dataset.default_as_union = false;
        self
    }

    /// Treats the union of named graphs as the default graph for USING datasets.
    #[must_use]
    pub fn default_graph_as_union(mut self) -> Self {
        self.dataset.default_as_union = true;
        self.dataset.default_graphs = None;
        self
    }

    /// Attaches a cooperative cancellation token (ADR-012).
    #[must_use]
    pub fn cancellation_token(mut self, token: CancellationToken) -> Self {
        self.cancellation = Some(token);
        self
    }

    /// Parses and executes the update against a model.
    ///
    /// On success, Fjall-backed models resync their durable copy from the
    /// in-memory store.
    pub fn execute(self, model: &Model) -> Result<()> {
        let mut evaluator = SparqlEvaluator::new();
        if let Some(base_iri) = &self.base_iri {
            evaluator = evaluator
                .with_base_iri(base_iri)
                .map_err(|error| Error::SparqlParse(error.to_string()))?;
        }
        for (name, iri) in &self.prefixes {
            evaluator = evaluator
                .with_prefix(name, iri)
                .map_err(|error| Error::SparqlParse(error.to_string()))?;
        }
        if let Some(token) = &self.cancellation {
            evaluator = evaluator.with_cancellation_token(token.clone());
        }
        let mut prepared = evaluator
            .parse_update(&self.text)
            .map_err(|error| Error::SparqlParse(error.to_string()))?;
        for dataset in prepared.using_datasets_mut() {
            apply_dataset(dataset, &self.dataset);
        }
        prepared
            .on_store(model.store())
            .execute()
            .map_err(|error| Error::SparqlEvaluation(error.to_string()))?;
        model.sync_disk_from_store()
    }
}

/// Closed set of SPARQL Query Results formats (ADR-011).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ResultsFormat {
    /// SPARQL Results XML.
    Xml,
    /// SPARQL Results JSON.
    Json,
    /// SPARQL Results CSV.
    Csv,
    /// SPARQL Results TSV.
    Tsv,
}

impl ResultsFormat {
    /// Canonical short name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Xml => "xml",
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Tsv => "tsv",
        }
    }

    /// Canonical media type.
    #[must_use]
    pub const fn media_type(self) -> &'static str {
        match self {
            Self::Xml => "application/sparql-results+xml",
            Self::Json => "application/sparql-results+json",
            Self::Csv => "text/csv",
            Self::Tsv => "text/tab-separated-values",
        }
    }

    /// Resolves a format name or common alias.
    pub fn from_name(name: &str) -> Result<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "xml" | "sparql-results+xml" => Ok(Self::Xml),
            "json" | "sparql-results+json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            "tsv" | "tab-separated-values" => Ok(Self::Tsv),
            other => Err(Error::Unsupported(format!(
                "SPARQL results format '{other}' is not advertised in Oxiland 0.3"
            ))),
        }
    }

    /// Resolves a media type (parameters ignored).
    pub fn from_media_type(media_type: &str) -> Result<Self> {
        let base = media_type
            .split(';')
            .next()
            .unwrap_or(media_type)
            .trim()
            .to_ascii_lowercase();
        match base.as_str() {
            "application/sparql-results+xml" => Ok(Self::Xml),
            "application/sparql-results+json" => Ok(Self::Json),
            "text/csv" => Ok(Self::Csv),
            "text/tab-separated-values" | "text/tsv" => Ok(Self::Tsv),
            other => Err(Error::Unsupported(format!(
                "SPARQL results media type '{other}' is not advertised in Oxiland 0.3"
            ))),
        }
    }

    fn to_oxigraph(self) -> QueryResultsFormat {
        match self {
            Self::Xml => QueryResultsFormat::Xml,
            Self::Json => QueryResultsFormat::Json,
            Self::Csv => QueryResultsFormat::Csv,
            Self::Tsv => QueryResultsFormat::Tsv,
        }
    }
}

/// Serializes ASK or SELECT [`QueryResults`] to a writer.
///
/// Graph results are not SPARQL Results documents—use [`crate::io::Serializer`].
pub fn serialize_query_results_to_writer<W: Write>(
    results: QueryResults<'_>,
    format: ResultsFormat,
    writer: W,
) -> Result<W> {
    let serializer = QueryResultsSerializer::from_format(format.to_oxigraph());
    match results {
        QueryResults::Boolean(value) => serializer
            .serialize_boolean_to_writer(writer, value)
            .map_err(|error| Error::Serialize(error.to_string())),
        QueryResults::Solutions(mut solutions) => {
            let variables = solutions.variables().to_vec();
            let mut solutions_writer = serializer
                .serialize_solutions_to_writer(writer, variables)
                .map_err(|error| Error::Serialize(error.to_string()))?;
            for solution in solutions.by_ref() {
                let solution =
                    solution.map_err(|error| Error::SparqlEvaluation(error.to_string()))?;
                solutions_writer
                    .serialize(&solution)
                    .map_err(|error| Error::Serialize(error.to_string()))?;
            }
            solutions_writer
                .finish()
                .map_err(|error| Error::Serialize(error.to_string()))
        }
        QueryResults::Graph(_) => Err(Error::Unsupported(
            "graph query results must be serialized with oxiland::io::Serializer, not ResultsFormat"
                .into(),
        )),
    }
}

/// Serializes ASK or SELECT [`QueryResults`] to a UTF-8 string.
pub fn serialize_query_results_to_string(
    results: QueryResults<'_>,
    format: ResultsFormat,
) -> Result<String> {
    let buffer = serialize_query_results_to_writer(results, format, Vec::new())?;
    String::from_utf8(buffer).map_err(|error| Error::Serialize(error.to_string()))
}

fn apply_slice(
    mut query: SparqlAlgebraQuery,
    offset: usize,
    limit: Option<usize>,
) -> Result<SparqlAlgebraQuery> {
    if offset == 0 && limit.is_none() {
        return Ok(query);
    }
    match &mut query {
        SparqlAlgebraQuery::Select { pattern, .. }
        | SparqlAlgebraQuery::Construct { pattern, .. }
        | SparqlAlgebraQuery::Describe { pattern, .. } => {
            let inner = std::mem::replace(
                pattern,
                GraphPattern::Bgp {
                    patterns: Vec::new(),
                },
            );
            *pattern = GraphPattern::Slice {
                inner: Box::new(inner),
                start: offset,
                length: limit,
            };
            Ok(query)
        }
        SparqlAlgebraQuery::Ask { .. } => Err(Error::Unsupported(
            "API-level limit/offset cannot be applied to ASK queries; put LIMIT in the query text if needed"
                .into(),
        )),
    }
}

fn apply_dataset(dataset: &mut oxigraph::sparql::QueryDataset, config: &DatasetConfig) {
    if config.default_as_union {
        dataset.set_default_graph_as_union();
    } else if let Some(graphs) = &config.default_graphs {
        dataset.set_default_graph(graphs.clone());
    }
    if let Some(named) = &config.named_graphs {
        dataset.set_available_named_graphs(named.clone());
    }
}
