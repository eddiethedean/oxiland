//! Redland `librdf_storage_*`-shaped facade over [`Model`] (0.10 parity).

use std::collections::BTreeSet;
use std::io::Write;

use oxigraph::model::{
    GraphName, GraphNameRef, NamedNode, NamedNodeRef, NamedOrBlankNode, NamedOrBlankNodeRef, Quad,
    Term, TermRef, Triple, TripleRef,
};

use crate::io::Serializer;
use crate::world::{BridgeToken, FeatureMap, FeatureValue};
use crate::{
    Error, Model, ModelTransaction, Query, QueryResults, Result, StatementMatches,
    StatementPattern, World,
};

/// Redland storage-method mirror over a [`Model`].
///
/// Oxiland does not expose a separate `DiskStore` type for every Redland
/// `librdf_storage_*` entry point. This facade forwards to [`Model`] so
/// inventory rows have a 1:1 safe mapping without duplicating storage engines.
#[derive(Clone)]
pub struct StorageFacade<'a> {
    model: &'a Model,
}

impl<'a> StorageFacade<'a> {
    /// Creates a storage view over `model`.
    #[must_use]
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }

    /// Returns the underlying model.
    #[must_use]
    pub fn model(&self) -> &'a Model {
        self.model
    }

    /// Redland `librdf_storage_add_reference` — no-op under Rust ownership.
    pub fn add_reference(&self) {}

    /// Redland `librdf_storage_remove_reference` — no-op under Rust ownership.
    pub fn remove_reference(&self) {}

    /// Redland `librdf_storage_close` — durable sync; models remain usable.
    pub fn close(&self) -> Result<()> {
        self.model.sync()
    }

    /// Statement count (`librdf_storage_size` / model length).
    pub fn size(&self) -> Result<usize> {
        self.model.len()
    }

    /// Adds a statement to the default graph.
    pub fn add_statement(&self, statement: impl Into<Triple>) -> Result<bool> {
        self.model.add(statement)
    }

    /// Adds many statements/quads (`librdf_storage_add_statements`).
    pub fn add_statements(&self, quads: impl IntoIterator<Item = Quad>) -> Result<usize> {
        self.model.bulk_insert_quads(quads)
    }

    /// Removes a statement from the default graph.
    pub fn remove_statement(&self, statement: impl Into<Triple>) -> Result<bool> {
        self.model.remove(statement)
    }

    /// Tests whether the default graph contains a statement.
    pub fn contains_statement(&self, statement: TripleRef<'_>) -> Result<bool> {
        self.model.contains(statement)
    }

    /// Adds a statement to a named context/graph.
    pub fn context_add_statement(
        &self,
        context: impl Into<GraphName>,
        statement: impl Into<Triple>,
    ) -> Result<bool> {
        self.model.add_to_graph(statement, context)
    }

    /// Adds many statements to a named context/graph.
    pub fn context_add_statements(
        &self,
        context: impl Into<GraphName>,
        statements: impl IntoIterator<Item = Triple>,
    ) -> Result<usize> {
        let context = context.into();
        let mut count = 0usize;
        for statement in statements {
            self.model.add_to_graph(statement, context.clone())?;
            count += 1;
        }
        Ok(count)
    }

    /// Removes a statement from a named context/graph.
    pub fn context_remove_statement(
        &self,
        context: impl Into<GraphName>,
        statement: impl Into<Triple>,
    ) -> Result<bool> {
        self.model.remove_from_graph(statement, context)
    }

    /// Clears a named context/graph (`librdf_storage_context_remove_statements`).
    pub fn context_remove_statements(&self, context: impl Into<GraphName>) -> Result<()> {
        self.model.clear_graph(context)
    }

    /// Streams statements in a named context.
    pub fn context_as_stream(&self, context: GraphNameRef<'_>) -> StatementMatches {
        self.model.find(StatementPattern {
            graph_name: Some(context),
            ..StatementPattern::default()
        })
    }

    /// Serializes a named context with `serializer`.
    pub fn context_serialise<W: Write>(
        &self,
        context: GraphNameRef<'_>,
        serializer: &Serializer,
        writer: W,
    ) -> Result<W> {
        let quads = self
            .context_as_stream(context)
            .collect::<Result<Vec<_>>>()?;
        serializer.serialize_quads_to_writer(writer, quads)
    }

    /// Finds statements matching a pattern.
    pub fn find_statements(&self, pattern: StatementPattern<'_>) -> StatementMatches {
        self.model.find(pattern)
    }

    /// Finds statements in a context matching subject/predicate/object.
    pub fn find_statements_in_context(
        &self,
        pattern: StatementPattern<'_>,
        context: GraphNameRef<'_>,
    ) -> StatementMatches {
        let mut narrowed = pattern;
        narrowed.graph_name = Some(context);
        self.model.find(narrowed)
    }

    /// Finds statements with options; options are currently unused and match
    /// [`Self::find_statements`].
    pub fn find_statements_with_options(
        &self,
        pattern: StatementPattern<'_>,
        _options: Option<&FeatureMap>,
    ) -> StatementMatches {
        self.find_statements(pattern)
    }

    /// Subjects matching `(?s, arc, target)` (`librdf_storage_get_sources`).
    pub fn get_sources(
        &self,
        arc: NamedNodeRef<'_>,
        target: TermRef<'_>,
    ) -> Result<Vec<NamedOrBlankNode>> {
        collect_subjects(self.model, None, Some(arc), Some(target))
    }

    /// Objects matching `(source, arc, ?o)` (`librdf_storage_get_targets`).
    pub fn get_targets(
        &self,
        source: NamedOrBlankNodeRef<'_>,
        arc: NamedNodeRef<'_>,
    ) -> Result<Vec<Term>> {
        collect_objects(self.model, Some(source), Some(arc), None)
    }

    /// Predicates matching `(source, ?p, target)` (`librdf_storage_get_arcs`).
    pub fn get_arcs(
        &self,
        source: NamedOrBlankNodeRef<'_>,
        target: TermRef<'_>,
    ) -> Result<Vec<NamedNode>> {
        collect_predicates(self.model, Some(source), None, Some(target))
    }

    /// Predicates into `node` (`librdf_storage_get_arcs_in`).
    pub fn get_arcs_in(&self, node: TermRef<'_>) -> Result<Vec<NamedNode>> {
        collect_predicates(self.model, None, None, Some(node))
    }

    /// Predicates out of `node` (`librdf_storage_get_arcs_out`).
    pub fn get_arcs_out(&self, node: NamedOrBlankNodeRef<'_>) -> Result<Vec<NamedNode>> {
        collect_predicates(self.model, Some(node), None, None)
    }

    /// Whether any arc points into `node`.
    pub fn has_arc_in(&self, node: TermRef<'_>) -> Result<bool> {
        Ok(!self.get_arcs_in(node)?.is_empty())
    }

    /// Whether any arc leaves `node`.
    pub fn has_arc_out(&self, node: NamedOrBlankNodeRef<'_>) -> Result<bool> {
        Ok(!self.get_arcs_out(node)?.is_empty())
    }

    /// Distinct named-graph contexts present in the store.
    pub fn get_contexts(&self) -> Result<Vec<GraphName>> {
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        for item in self.model.find(StatementPattern::default()) {
            let quad = item?;
            if matches!(quad.graph_name, GraphName::DefaultGraph) {
                continue;
            }
            let key = quad.graph_name.to_string();
            if seen.insert(key) {
                out.push(quad.graph_name);
            }
        }
        Ok(out)
    }

    /// Storage feature get (`librdf_storage_get_feature`).
    #[must_use]
    pub fn feature(&self, iri: &str) -> Option<FeatureValue> {
        self.model.storage_feature(iri)
    }

    /// Storage feature set (`librdf_storage_set_feature`).
    pub fn set_feature(&self, iri: impl Into<String>, value: FeatureValue) {
        self.model.set_storage_feature(iri, value);
    }

    /// Opaque instance token (`librdf_storage_get_instance`).
    #[must_use]
    pub fn instance(&self) -> Option<BridgeToken> {
        self.model.storage_instance()
    }

    /// Opaque instance token (`librdf_storage_set_instance`).
    pub fn set_instance(&self, token: Option<BridgeToken>) {
        self.model.set_storage_instance(token);
    }

    /// Associated world (`librdf_storage_get_world`).
    #[must_use]
    pub fn world(&self) -> &World {
        self.model.world()
    }

    /// Executes a SPARQL query against the model.
    pub fn query_execute<'m>(&'m self, query: &Query) -> Result<QueryResults<'m>> {
        query.execute(self.model)
    }

    /// Serializes the whole store.
    pub fn serialise<W: Write>(&self, serializer: &Serializer, writer: W) -> Result<W> {
        serializer.serialize_model_to_writer(writer, self.model)
    }

    /// Runs a transaction (`librdf_storage_transaction_*` RAII mapping).
    pub fn transaction<R>(
        &self,
        f: impl FnOnce(&mut ModelTransaction<'_>) -> Result<R>,
    ) -> Result<R> {
        self.model.transaction(f)
    }

    /// Starts a transaction; commit is implicit on success of `f`.
    pub fn transaction_start<R>(
        &self,
        f: impl FnOnce(&mut ModelTransaction<'_>) -> Result<R>,
    ) -> Result<R> {
        self.transaction(f)
    }

    /// Starts a transaction, ignoring a Redland-style external handle token.
    pub fn transaction_start_with_handle<R>(
        &self,
        _handle: Option<BridgeToken>,
        f: impl FnOnce(&mut ModelTransaction<'_>) -> Result<R>,
    ) -> Result<R> {
        self.transaction(f)
    }

    /// No separate transaction handle exists under RAII (`None`).
    #[must_use]
    pub fn transaction_get_handle(&self) -> Option<BridgeToken> {
        None
    }

    /// Explicit commit outside [`Self::transaction`] is a documented no-op:
    /// commits happen when the transaction callback returns successfully.
    pub fn transaction_commit(&self) -> Result<()> {
        Ok(())
    }

    /// Explicit rollback outside a callback is unsupported.
    pub fn transaction_rollback(&self) -> Result<()> {
        Err(Error::Unsupported(
            "storage transaction rollback outside Model::transaction is unsupported; return Err from the transaction callback to abort"
                .into(),
        ))
    }

    /// Durable sync.
    pub fn sync(&self) -> Result<()> {
        self.model.sync()
    }
}

fn collect_subjects(
    model: &Model,
    subject: Option<NamedOrBlankNodeRef<'_>>,
    predicate: Option<NamedNodeRef<'_>>,
    object: Option<TermRef<'_>>,
) -> Result<Vec<NamedOrBlankNode>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for item in model.find(StatementPattern {
        subject,
        predicate,
        object,
        graph_name: None,
    }) {
        let quad = item?;
        let key = quad.subject.to_string();
        if seen.insert(key) {
            out.push(quad.subject);
        }
    }
    Ok(out)
}

fn collect_objects(
    model: &Model,
    subject: Option<NamedOrBlankNodeRef<'_>>,
    predicate: Option<NamedNodeRef<'_>>,
    object: Option<TermRef<'_>>,
) -> Result<Vec<Term>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for item in model.find(StatementPattern {
        subject,
        predicate,
        object,
        graph_name: None,
    }) {
        let quad = item?;
        let key = quad.object.to_string();
        if seen.insert(key) {
            out.push(quad.object);
        }
    }
    Ok(out)
}

fn collect_predicates(
    model: &Model,
    subject: Option<NamedOrBlankNodeRef<'_>>,
    predicate: Option<NamedNodeRef<'_>>,
    object: Option<TermRef<'_>>,
) -> Result<Vec<NamedNode>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for item in model.find(StatementPattern {
        subject,
        predicate,
        object,
        graph_name: None,
    }) {
        let quad = item?;
        let key = quad.predicate.as_str().to_owned();
        if seen.insert(key) {
            out.push(quad.predicate);
        }
    }
    Ok(out)
}
