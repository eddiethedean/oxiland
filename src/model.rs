#[cfg(feature = "rocksdb")]
use std::path::Path;

use oxigraph::model::{
    GraphName, GraphNameRef, NamedOrBlankNodeRef, Quad, QuadRef, TermRef, Triple, TripleRef,
};
use oxigraph::store::Store;

use crate::{Error, Result};

/// A partial triple pattern, equivalent to Redland's statement matching API.
#[derive(Clone, Copy, Debug, Default)]
pub struct StatementPattern<'a> {
    /// Optional subject constraint.
    pub subject: Option<NamedOrBlankNodeRef<'a>>,
    /// Optional predicate constraint.
    pub predicate: Option<oxigraph::model::NamedNodeRef<'a>>,
    /// Optional object constraint.
    pub object: Option<TermRef<'a>>,
    /// Optional graph/context constraint. `None` searches all contexts.
    pub graph_name: Option<GraphNameRef<'a>>,
}

/// An RDF graph model backed by an Oxigraph store.
#[derive(Clone)]
pub struct Model {
    store: Store,
}

impl Model {
    /// Creates an empty in-memory model.
    pub fn new() -> Result<Self> {
        Store::new()
            .map(|store| Self { store })
            .map_err(|error| Error::Storage(error.to_string()))
    }

    /// Opens or creates a persistent RocksDB model.
    #[cfg(feature = "rocksdb")]
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        Store::open(path)
            .map(|store| Self { store })
            .map_err(|error| Error::OpenStore {
                path: path.to_owned(),
                message: error.to_string(),
            })
    }

    /// Returns the underlying Oxigraph store.
    #[must_use]
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Adds a statement to the default graph.
    pub fn add(&self, statement: impl Into<Triple>) -> Result<bool> {
        self.add_to_graph(statement, GraphName::DefaultGraph)
    }

    /// Adds a statement to a named graph/context.
    pub fn add_to_graph(
        &self,
        statement: impl Into<Triple>,
        graph_name: impl Into<GraphName>,
    ) -> Result<bool> {
        let triple = statement.into();
        let quad = Quad::new(triple.subject, triple.predicate, triple.object, graph_name);
        let inserted = !self
            .store
            .contains(quad.as_ref())
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.store
            .insert(&quad)
            .map_err(|error| Error::Storage(error.to_string()))?;
        Ok(inserted)
    }

    /// Removes a statement from the default graph.
    pub fn remove(&self, statement: impl Into<Triple>) -> Result<bool> {
        let triple = statement.into();
        let quad = QuadRef::new(
            triple.subject.as_ref(),
            triple.predicate.as_ref(),
            triple.object.as_ref(),
            GraphNameRef::DefaultGraph,
        );
        let removed = self
            .store
            .contains(quad)
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.store
            .remove(quad)
            .map_err(|error| Error::Storage(error.to_string()))?;
        Ok(removed)
    }

    /// Tests whether the default graph contains a statement.
    pub fn contains(&self, statement: TripleRef<'_>) -> Result<bool> {
        self.store
            .contains(oxigraph::model::QuadRef::new(
                statement.subject,
                statement.predicate,
                statement.object,
                GraphNameRef::DefaultGraph,
            ))
            .map_err(|error| Error::Storage(error.to_string()))
    }

    /// Returns the number of statements across all contexts.
    pub fn len(&self) -> Result<usize> {
        self.store
            .len()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    /// Returns whether the model contains no statements.
    pub fn is_empty(&self) -> Result<bool> {
        self.store
            .is_empty()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    /// Finds quads matching a partial statement/context pattern.
    pub fn find(&self, pattern: StatementPattern<'_>) -> Result<Vec<Quad>> {
        self.store
            .quads_for_pattern(
                pattern.subject,
                pattern.predicate,
                pattern.object,
                pattern.graph_name,
            )
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| Error::Storage(error.to_string()))
    }
}
