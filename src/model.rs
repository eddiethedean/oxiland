use std::path::Path;
use std::sync::{Arc, Mutex};

use oxigraph::model::{
    GraphName, GraphNameRef, NamedOrBlankNodeRef, Quad, TermRef, Triple, TripleRef,
};
use oxigraph::store::{QuadIter, Store};

use crate::persist::DiskStore;
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

/// Streaming matches produced by [`Model::find`].
///
/// The iterator yields owned [`Quad`] values from a store snapshot and does not
/// borrow the [`Model`]. Errors from the storage backend surface as
/// [`Error::Storage`].
#[must_use]
pub struct StatementMatches {
    inner: QuadIter<'static>,
}

impl Iterator for StatementMatches {
    type Item = Result<Quad>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|item| item.map_err(|error| Error::Storage(error.to_string())))
    }
}

/// An RDF graph model backed by an Oxigraph store.
///
/// In-memory models use Oxigraph alone. Persistent models opened with
/// [`Model::open`] keep an Oxigraph working set and a
/// [Fjall](https://github.com/fjall-rs/fjall) durable copy of every quad.
///
/// Cloning a [`Model`] clones the store handle and shares the same dataset; it
/// does not deep-copy statements. `Model` is `Send` and `Sync`.
///
/// Mutating methods (`add` / `remove` / `insert_quad` / …) serialize through an
/// internal lock so the `bool` “newly inserted / removed” return value stays
/// accurate under concurrent callers that share a cloned handle.
///
/// # Examples
///
/// ```
/// use oxiland::terms::{self, Literal, Triple};
/// use oxiland::{Model, StatementPattern};
///
/// # fn main() -> oxiland::Result<()> {
/// let model = Model::new()?;
/// let statement = Triple::new(
///     terms::named_node("https://example.com/alice")?,
///     terms::named_node("https://example.com/name")?,
///     Literal::new_simple_literal("Alice"),
/// );
///
/// assert!(model.add(statement.clone())?);
/// assert!(model.contains(statement.as_ref())?);
///
/// let matches = model
///     .find(StatementPattern {
///         subject: Some(statement.subject.as_ref()),
///         ..StatementPattern::default()
///     })
///     .collect::<Result<Vec<_>, _>>()?;
/// assert_eq!(matches.len(), 1);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Model {
    store: Store,
    disk: Option<DiskStore>,
    write_lock: Arc<Mutex<()>>,
}

impl Model {
    /// Creates an empty in-memory model.
    pub fn new() -> Result<Self> {
        Store::new()
            .map(|store| Self {
                store,
                disk: None,
                write_lock: Arc::new(Mutex::new(())),
            })
            .map_err(|error| Error::Storage(error.to_string()))
    }

    /// Opens or creates a persistent model at `path`.
    ///
    /// Quads are stored in a Fjall keyspace and loaded into an Oxigraph
    /// in-memory working set for querying. On-disk format compatibility across
    /// Oxiland versions is not guaranteed in 0.x; see ADR-006.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let disk = DiskStore::open(path)?;
        let store = Store::new().map_err(|error| Error::OpenStore {
            path: path.to_owned(),
            message: error.to_string(),
        })?;
        disk.load_into(&store)?;
        Ok(Self {
            store,
            disk: Some(disk),
            write_lock: Arc::new(Mutex::new(())),
        })
    }

    /// Returns the underlying Oxigraph store.
    #[must_use]
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Reports whether a named storage backend is available in this build.
    ///
    /// Unknown backend names return [`Error::Unsupported`].
    pub fn storage_backend_available(name: &str) -> Result<bool> {
        match name {
            "memory" | "fjall" => Ok(true),
            "rocksdb" | "redb" => Err(Error::Unsupported(
                "storage backend was replaced by fjall; use Model::open".into(),
            )),
            other => Err(Error::Unsupported(format!(
                "storage backend '{other}' is not recognized"
            ))),
        }
    }

    /// Adds a statement to the default graph.
    ///
    /// Returns `true` when the statement was newly inserted and `false` when it
    /// was already present.
    pub fn add(&self, statement: impl Into<Triple>) -> Result<bool> {
        self.add_to_graph(statement, GraphName::DefaultGraph)
    }

    /// Adds a statement to a named graph/context.
    ///
    /// Returns `true` when the statement was newly inserted and `false` when it
    /// was already present in that graph.
    pub fn add_to_graph(
        &self,
        statement: impl Into<Triple>,
        graph_name: impl Into<GraphName>,
    ) -> Result<bool> {
        let triple = statement.into();
        let quad = Quad::new(triple.subject, triple.predicate, triple.object, graph_name);
        self.insert_quad(quad)
    }

    /// Inserts a fully formed quad into the model.
    ///
    /// Returns `true` when the quad was newly inserted and `false` when it was
    /// already present. Used by progressive parser loads (ADR-007).
    pub fn insert_quad(&self, quad: Quad) -> Result<bool> {
        let _guard = self
            .write_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let inserted = !self
            .store
            .contains(quad.as_ref())
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.store
            .insert(&quad)
            .map_err(|error| Error::Storage(error.to_string()))?;
        if let Some(disk) = &self.disk {
            if let Err(error) = disk.insert(&quad) {
                // Only roll back quads this call newly inserted. Removing when
                // `inserted == false` would delete a pre-existing statement.
                if inserted {
                    let _ = self.store.remove(quad.as_ref());
                }
                return Err(error);
            }
        }
        Ok(inserted)
    }

    /// Removes a fully formed quad from the model.
    ///
    /// Returns `true` when a matching quad was removed.
    pub fn remove_quad(&self, quad: &Quad) -> Result<bool> {
        let _guard = self
            .write_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let removed = self
            .store
            .contains(quad.as_ref())
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.store
            .remove(quad.as_ref())
            .map_err(|error| Error::Storage(error.to_string()))?;
        if removed {
            if let Some(disk) = &self.disk {
                if let Err(error) = disk.remove(quad) {
                    let _ = self.store.insert(quad);
                    return Err(error);
                }
            }
        }
        Ok(removed)
    }

    /// Removes a statement from the default graph.
    ///
    /// Returns `true` when a matching statement was removed.
    pub fn remove(&self, statement: impl Into<Triple>) -> Result<bool> {
        self.remove_from_graph(statement, GraphName::DefaultGraph)
    }

    /// Removes a statement from a named graph/context.
    ///
    /// Returns `true` when a matching statement was removed from that graph.
    pub fn remove_from_graph(
        &self,
        statement: impl Into<Triple>,
        graph_name: impl Into<GraphName>,
    ) -> Result<bool> {
        let triple = statement.into();
        let quad = Quad::new(triple.subject, triple.predicate, triple.object, graph_name);
        self.remove_quad(&quad)
    }

    /// Tests whether the default graph contains a statement.
    pub fn contains(&self, statement: TripleRef<'_>) -> Result<bool> {
        self.contains_in_graph(statement, GraphNameRef::DefaultGraph)
    }

    /// Tests whether a named graph/context contains a statement.
    pub fn contains_in_graph(
        &self,
        statement: TripleRef<'_>,
        graph_name: GraphNameRef<'_>,
    ) -> Result<bool> {
        self.store
            .contains(oxigraph::model::QuadRef::new(
                statement.subject,
                statement.predicate,
                statement.object,
                graph_name,
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

    /// Streams quads matching a partial statement/context pattern.
    ///
    /// Matching uses a store snapshot and yields results lazily. Callers can
    /// stop early without materializing the full match set.
    pub fn find(&self, pattern: StatementPattern<'_>) -> StatementMatches {
        StatementMatches {
            inner: self.store.quads_for_pattern(
                pattern.subject,
                pattern.predicate,
                pattern.object,
                pattern.graph_name,
            ),
        }
    }

    /// After SPARQL Update mutates the Oxigraph store, resync Fjall if present.
    pub(crate) fn sync_disk_from_store(&self) -> Result<()> {
        let Some(disk) = &self.disk else {
            return Ok(());
        };
        let _guard = self
            .write_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        disk.replace_all_from_store(&self.store)
    }
}
