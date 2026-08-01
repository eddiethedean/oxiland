use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::ThreadId;

use oxigraph::io::{RdfFormat, RdfParser, RdfSerializer};
use oxigraph::model::{
    GraphName, GraphNameRef, NamedOrBlankNodeRef, Quad, QuadRef, TermRef, Triple, TripleRef,
};
use oxigraph::store::{QuadIter, Store, Transaction as OxigraphTransaction};

use crate::io::{BomStrippingReader, map_rdf_parse_error};
use crate::storage::{
    self, DurableStore, DurableStoreOps, OpenOptions, StorageBackend, StorageCapabilities,
    StorageFacade,
};
use crate::world::{BridgeToken, FeatureMap, FeatureValue, World};
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

/// Mutator available inside [`Model::transaction`].
pub struct ModelTransaction<'a> {
    inner: OxigraphTransaction<'a>,
}

impl ModelTransaction<'_> {
    /// Adds a statement to the default graph within the transaction.
    pub fn add(&mut self, statement: impl Into<Triple>) -> Result<bool> {
        self.add_to_graph(statement, GraphName::DefaultGraph)
    }

    /// Adds a statement to a named graph within the transaction.
    pub fn add_to_graph(
        &mut self,
        statement: impl Into<Triple>,
        graph_name: impl Into<GraphName>,
    ) -> Result<bool> {
        let triple = statement.into();
        let quad = Quad::new(triple.subject, triple.predicate, triple.object, graph_name);
        self.insert_quad(quad)
    }

    /// Inserts a quad within the transaction.
    pub fn insert_quad(&mut self, quad: Quad) -> Result<bool> {
        let inserted = !self
            .inner
            .contains(quad.as_ref())
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.inner.insert(quad.as_ref());
        Ok(inserted)
    }

    /// Removes a statement from the default graph within the transaction.
    pub fn remove(&mut self, statement: impl Into<Triple>) -> Result<bool> {
        self.remove_from_graph(statement, GraphName::DefaultGraph)
    }

    /// Removes a statement from a named graph within the transaction.
    pub fn remove_from_graph(
        &mut self,
        statement: impl Into<Triple>,
        graph_name: impl Into<GraphName>,
    ) -> Result<bool> {
        let triple = statement.into();
        let quad = Quad::new(triple.subject, triple.predicate, triple.object, graph_name);
        self.remove_quad(&quad)
    }

    /// Removes a quad within the transaction.
    pub fn remove_quad(&mut self, quad: &Quad) -> Result<bool> {
        let removed = self
            .inner
            .contains(quad.as_ref())
            .map_err(|error| Error::Storage(error.to_string()))?;
        if removed {
            self.inner.remove(quad.as_ref());
        }
        Ok(removed)
    }

    /// Clears the entire dataset within the transaction.
    pub fn clear(&mut self) -> Result<()> {
        self.inner
            .clear()
            .map_err(|error| Error::Storage(error.to_string()))
    }

    /// Clears one graph within the transaction.
    pub fn clear_graph(&mut self, graph_name: impl Into<GraphName>) -> Result<()> {
        let graph_name = graph_name.into();
        self.inner
            .clear_graph(graph_name.as_ref())
            .map_err(|error| Error::Storage(error.to_string()))
    }
}

/// An RDF graph model backed by an Oxigraph store.
///
/// In-memory models use Oxigraph alone. Persistent models opened with
/// [`Model::open`] / [`Model::open_with`] keep an Oxigraph working set and a
/// Fjall durable copy under Oxiland format v1 (ADR-006).
///
/// Cloning a [`Model`] clones the store handle and shares the same dataset; it
/// does not deep-copy statements. `Model` is `Send` and `Sync`.
///
/// Readers (`find` / query execution) take a shared lock; writers take an
/// exclusive lock so Fjall reload cannot expose an empty working set.
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
    disk: Option<DurableStore>,
    lock: Arc<RwLock<()>>,
    read_only: bool,
    in_transaction: Arc<AtomicBool>,
    txn_owner: Arc<Mutex<Option<ThreadId>>>,
    world: World,
    feature_map: FeatureMap,
    storage_features: FeatureMap,
    storage_instance: Arc<RwLock<Option<BridgeToken>>>,
}

struct InTransactionGuard<'a> {
    model: &'a Model,
}

impl Drop for InTransactionGuard<'_> {
    fn drop(&mut self) {
        self.model.in_transaction.store(false, Ordering::Release);
        *self
            .model
            .txn_owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }
}

impl Model {
    /// Creates an empty in-memory model with a fresh [`World`].
    pub fn new() -> Result<Self> {
        Self::with_world(World::new())
    }

    /// Creates an empty in-memory model associated with `world`.
    pub fn with_world(world: World) -> Result<Self> {
        Store::new()
            .map(|store| Self::from_parts(store, None, false, world))
            .map_err(|error| Error::Storage(error.to_string()))
    }

    fn from_parts(store: Store, disk: Option<DurableStore>, read_only: bool, world: World) -> Self {
        Self {
            store,
            disk,
            lock: Arc::new(RwLock::new(())),
            read_only,
            in_transaction: Arc::new(AtomicBool::new(false)),
            txn_owner: Arc::new(Mutex::new(None)),
            world,
            feature_map: FeatureMap::new(),
            storage_features: FeatureMap::new(),
            storage_instance: Arc::new(RwLock::new(None)),
        }
    }

    /// Returns the associated [`World`].
    #[must_use]
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Assigns a model feature (`librdf_model_set_feature`).
    pub fn set_feature(&self, iri: impl Into<String>, value: FeatureValue) {
        self.feature_map.set(iri, value);
    }

    /// Returns a model feature (`librdf_model_get_feature`).
    #[must_use]
    pub fn feature(&self, iri: &str) -> Option<FeatureValue> {
        self.feature_map.get(iri)
    }

    /// Assigns a storage feature used by [`crate::storage::StorageFacade`].
    pub fn set_storage_feature(&self, iri: impl Into<String>, value: FeatureValue) {
        self.storage_features.set(iri, value);
    }

    /// Returns a storage feature.
    #[must_use]
    pub fn storage_feature(&self, iri: &str) -> Option<FeatureValue> {
        self.storage_features.get(iri)
    }

    /// Opaque storage instance token (`librdf_storage_get/set_instance`).
    #[must_use]
    pub fn storage_instance(&self) -> Option<BridgeToken> {
        *self
            .storage_instance
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Sets the opaque storage instance token.
    pub fn set_storage_instance(&self, token: Option<BridgeToken>) {
        *self
            .storage_instance
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = token;
    }

    /// Returns a Redland `librdf_storage_*`-shaped view of this model.
    #[must_use]
    pub fn as_storage(&self) -> StorageFacade<'_> {
        StorageFacade::new(self)
    }

    /// Opens or creates a persistent format-v1 model at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with(OpenOptions::fjall(path))
    }

    /// Opens a persistent model with typed options (ADR-006 / ADR-022).
    pub fn open_with(options: OpenOptions) -> Result<Self> {
        if options.backend() == StorageBackend::Memory {
            return Self::new();
        }

        let path = options.path();
        if options.is_read_only() {
            if !path.exists() {
                return Err(Error::OpenStore {
                    path: path.to_owned(),
                    message: "read-only open requires an existing store path".into(),
                });
            }
            if !DurableStore::looks_like_store(options.backend(), path) {
                return Err(Error::OpenStore {
                    path: path.to_owned(),
                    message: "read-only open cannot initialize a new Oxiland store".into(),
                });
            }
        }

        let disk = DurableStore::open(options.backend(), path, options.can_create())?;
        let allow_init = !options.is_read_only() && options.can_create();
        disk.ensure_format_v1(path, allow_init)?;
        let store = Store::new().map_err(|error| Error::OpenStore {
            path: path.to_owned(),
            message: error.to_string(),
        })?;
        disk.load_into(&store)?;
        Ok(Self::from_parts(
            store,
            Some(disk),
            options.is_read_only(),
            World::new(),
        ))
    }

    /// Copies this model into a newly created destination store.
    ///
    /// The destination must not already look like an Oxiland store for the
    /// requested backend. On failure the destination directory is left in a
    /// state that is safe to delete; the source is never rewritten in place.
    pub fn copy_to(&self, options: OpenOptions) -> Result<Self> {
        if options.backend() == StorageBackend::Memory {
            let dest = Self::new()?;
            for quad in self.find(StatementPattern::default()) {
                dest.insert_quad(quad?)?;
            }
            return Ok(dest);
        }
        if options.is_read_only() {
            return Err(Error::Unsupported(
                "Model::copy_to requires a writable destination".into(),
            ));
        }
        if !options.can_create() {
            return Err(Error::Unsupported(
                "Model::copy_to requires OpenOptions::create(true)".into(),
            ));
        }
        let path = options.path();
        if DurableStore::looks_like_store(options.backend(), path) {
            return Err(Error::OpenStore {
                path: path.to_owned(),
                message:
                    "copy_to destination already looks like an Oxiland store; refuse to overwrite"
                        .into(),
            });
        }
        let dest = Self::open_with(options)?;
        for quad in self.find(StatementPattern::default()) {
            dest.insert_quad(quad?)?;
        }
        dest.sync()?;
        Ok(dest)
    }

    /// Migrates a pre-0.4 experimental Fjall directory to format v1, then opens it.
    pub fn migrate_legacy_store(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        {
            let disk = DurableStore::open(StorageBackend::Fjall, path, false)?;
            disk.migrate_legacy_to_v1()?;
        }
        Self::open_with(OpenOptions::fjall(path))
    }

    /// Returns capability bits for this model.
    #[must_use]
    pub fn capabilities(&self) -> StorageCapabilities {
        match &self.disk {
            None => StorageCapabilities::memory(),
            Some(disk) => disk.capabilities(self.read_only),
        }
    }

    /// Returns the storage backend for this model.
    #[must_use]
    pub fn backend(&self) -> StorageBackend {
        match &self.disk {
            None => StorageBackend::Memory,
            Some(disk) => disk.backend_id(),
        }
    }

    /// Returns the underlying Oxigraph store.
    ///
    /// This is an escape hatch for advanced Oxigraph use. Mutations through the
    /// returned handle bypass Oxiland's lock and Fjall durability sync.
    /// Prefer [`Model::insert_quad`], [`Model::transaction`], and
    /// [`crate::Update`] so memory and disk stay aligned.
    #[must_use]
    pub fn store(&self) -> &Store {
        &self.store
    }

    pub(crate) fn with_read_lock<R>(&self, f: impl FnOnce() -> R) -> R {
        if self.same_thread_in_transaction() {
            // Avoid deadlocking on the non-reentrant RwLock held by transaction().
            // Reads see the committed working set, not uncommitted txn mutations.
            return f();
        }
        let _guard = self
            .lock
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f()
    }

    /// Reports whether a named storage backend is available in this build.
    pub fn storage_backend_available(name: &str) -> Result<bool> {
        let backend = StorageBackend::from_name(name)?;
        Ok(storage::compiled_backends().contains(&backend))
    }

    fn same_thread_in_transaction(&self) -> bool {
        if !self.in_transaction.load(Ordering::Acquire) {
            return false;
        }
        let owner = self
            .txn_owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *owner == Some(std::thread::current().id())
    }

    fn ensure_writable(&self) -> Result<()> {
        if self.read_only {
            return Err(Error::Unsupported(
                "model was opened read-only; mutating APIs are unavailable".into(),
            ));
        }
        if self.in_transaction.load(Ordering::Acquire) {
            return Err(Error::Unsupported(
                "auto-commit mutation is unavailable while a Model::transaction is open; use the transaction handle"
                    .into(),
            ));
        }
        Ok(())
    }

    /// Runs `f` inside an Oxigraph transaction; Fjall models sync on commit.
    ///
    /// Same-thread `Model` reads (`len` / `find` / `Query::execute`) during the
    /// callback see the last committed working set and do not deadlock. Use
    /// [`ModelTransaction`] methods for in-transaction mutations. Nested
    /// `transaction` / auto-commit writes return [`Error::Unsupported`].
    pub fn transaction<R>(
        &self,
        f: impl FnOnce(&mut ModelTransaction<'_>) -> Result<R>,
    ) -> Result<R> {
        if self.read_only {
            return Err(Error::Unsupported(
                "model was opened read-only; mutating APIs are unavailable".into(),
            ));
        }
        if self.same_thread_in_transaction() {
            return Err(Error::Unsupported(
                "nested Model::transaction is unsupported".into(),
            ));
        }
        let _guard = self
            .lock
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self
            .in_transaction
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(Error::Unsupported(
                "nested Model::transaction is unsupported".into(),
            ));
        }
        *self
            .txn_owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(std::thread::current().id());
        let _txn_flag = InTransactionGuard { model: self };
        let oxi = self
            .store
            .start_transaction()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let mut tx = ModelTransaction { inner: oxi };
        let value = f(&mut tx)?;
        let ModelTransaction { inner } = tx;
        inner
            .commit()
            .map_err(|error| Error::Storage(error.to_string()))?;
        if let Some(disk) = &self.disk {
            if let Err(error) = disk.replace_all_from_store(&self.store) {
                if let Err(reload_error) = self.reload_store_from_disk_unlocked(disk) {
                    return Err(Error::Storage(format!(
                        "durable sync failed after transaction ({error}); rollback from disk also failed ({reload_error})"
                    )));
                }
                return Err(Error::Storage(format!(
                    "durable sync failed after transaction; in-memory store rolled back to disk: {error}"
                )));
            }
        }
        Ok(value)
    }

    /// Forces a durable sync (Fjall `SyncAll`). No-op success for memory models.
    pub fn sync(&self) -> Result<()> {
        if self.same_thread_in_transaction() {
            return Err(Error::Unsupported(
                "sync is unavailable while a Model::transaction is open on this thread".into(),
            ));
        }
        let _guard = self
            .lock
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match &self.disk {
            Some(disk) => disk.sync(),
            None => Ok(()),
        }
    }

    /// Clears all statements (and named graphs) from the model.
    pub fn clear(&self) -> Result<()> {
        self.ensure_writable()?;
        let _guard = self
            .lock
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.store
            .clear()
            .map_err(|error| Error::Storage(error.to_string()))?;
        if let Some(disk) = &self.disk {
            if let Err(error) = disk.clear_quads() {
                if let Err(reload_error) = self.reload_store_from_disk_unlocked(disk) {
                    return Err(Error::Storage(format!(
                        "durable clear failed ({error}); rollback from disk also failed ({reload_error})"
                    )));
                }
                return Err(error);
            }
        }
        Ok(())
    }

    /// Clears a single graph/context.
    pub fn clear_graph(&self, graph_name: impl Into<GraphName>) -> Result<()> {
        self.ensure_writable()?;
        let graph_name = graph_name.into();
        let _guard = self
            .lock
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.store
            .clear_graph(graph_name.as_ref())
            .map_err(|error| Error::Storage(error.to_string()))?;
        if let Some(disk) = &self.disk {
            if let Err(error) = disk.replace_all_from_store(&self.store) {
                if let Err(reload_error) = self.reload_store_from_disk_unlocked(disk) {
                    return Err(Error::Storage(format!(
                        "durable clear_graph failed ({error}); rollback from disk also failed ({reload_error})"
                    )));
                }
                return Err(error);
            }
        }
        Ok(())
    }

    /// Inserts many quads inside a single transaction (then durable sync).
    ///
    /// Returns the number of quads in the input iterator (including duplicates
    /// that were already present), not the count of newly inserted quads.
    pub fn bulk_insert_quads(&self, quads: impl IntoIterator<Item = Quad>) -> Result<usize> {
        let quads: Vec<_> = quads.into_iter().collect();
        let total = quads.len();
        self.transaction(|tx| {
            for quad in quads {
                tx.insert_quad(quad)?;
            }
            Ok(total)
        })
    }

    /// Exports the model as N-Quads to a filesystem path (archival helper).
    pub fn export_nquads_to_path(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let file = File::create(path).map_err(|error| {
            Error::Io(std::io::Error::new(
                error.kind(),
                format!("{}: {}", path.display(), error),
            ))
        })?;
        let mut serializer =
            RdfSerializer::from_format(RdfFormat::NQuads).for_writer(BufWriter::new(file));
        for item in self.find(StatementPattern::default()) {
            let quad = item?;
            serializer
                .serialize_quad(QuadRef::from(&quad))
                .map_err(Error::Io)?;
        }
        let writer = serializer.finish().map_err(Error::Io)?;
        writer
            .into_inner()
            .map_err(|error| Error::Io(error.into_error()))?;
        Ok(())
    }

    /// Imports N-Quads from a path inside a transaction (atomic on success).
    ///
    /// Quads are **merged** into the existing model (RDF union); this does not
    /// clear the store first. A leading UTF-8 BOM is skipped when present.
    pub fn import_nquads_from_path(&self, path: impl AsRef<Path>) -> Result<usize> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|error| {
            Error::Io(std::io::Error::new(
                error.kind(),
                format!("{}: {}", path.display(), error),
            ))
        })?;
        let reader = BomStrippingReader::new(BufReader::new(file));
        let quads = RdfParser::from_format(RdfFormat::NQuads)
            .rename_blank_nodes()
            .for_reader(reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(map_rdf_parse_error)?;
        let total = quads.len();
        self.bulk_insert_quads(quads)?;
        Ok(total)
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
        self.insert_quad(quad)
    }

    /// Inserts a fully formed quad into the model.
    pub fn insert_quad(&self, quad: Quad) -> Result<bool> {
        self.ensure_writable()?;
        let _guard = self
            .lock
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let inserted = !self
            .store
            .contains(quad.as_ref())
            .map_err(|error| Error::Storage(error.to_string()))?;
        if !inserted {
            return Ok(false);
        }
        self.store
            .insert(&quad)
            .map_err(|error| Error::Storage(error.to_string()))?;
        if let Some(disk) = &self.disk {
            let canonical = storage::stored_matching_quad(&self.store, &quad)?;
            if let Err(error) = disk.insert_quad(&canonical) {
                if let Err(reload_error) = self.reload_store_from_disk_unlocked(disk) {
                    let _ = self.store.remove(canonical.as_ref());
                    return Err(Error::Storage(format!(
                        "durable insert failed ({error}); rollback from disk also failed ({reload_error})"
                    )));
                }
                return Err(error);
            }
        }
        Ok(true)
    }

    /// Removes a fully formed quad from the model.
    pub fn remove_quad(&self, quad: &Quad) -> Result<bool> {
        self.ensure_writable()?;
        let _guard = self
            .lock
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let removed = self
            .store
            .contains(quad.as_ref())
            .map_err(|error| Error::Storage(error.to_string()))?;
        if !removed {
            return Ok(false);
        }
        let canonical = storage::stored_matching_quad(&self.store, quad)?;
        self.store
            .remove(quad.as_ref())
            .map_err(|error| Error::Storage(error.to_string()))?;
        if let Some(disk) = &self.disk {
            if let Err(error) = disk.remove_rdf_equal(&canonical) {
                if let Err(reload_error) = self.reload_store_from_disk_unlocked(disk) {
                    let _ = self.store.insert(&canonical);
                    return Err(Error::Storage(format!(
                        "durable remove failed ({error}); rollback from disk also failed ({reload_error})"
                    )));
                }
                return Err(error);
            }
        }
        Ok(true)
    }

    /// Removes a statement from the default graph.
    pub fn remove(&self, statement: impl Into<Triple>) -> Result<bool> {
        self.remove_from_graph(statement, GraphName::DefaultGraph)
    }

    /// Removes a statement from a named graph/context.
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
        self.with_read_lock(|| {
            self.store
                .contains(oxigraph::model::QuadRef::new(
                    statement.subject,
                    statement.predicate,
                    statement.object,
                    graph_name,
                ))
                .map_err(|error| Error::Storage(error.to_string()))
        })
    }

    /// Returns the number of statements across all contexts.
    pub fn len(&self) -> Result<usize> {
        self.with_read_lock(|| {
            self.store
                .len()
                .map_err(|error| Error::Storage(error.to_string()))
        })
    }

    /// Returns whether the model contains no statements.
    pub fn is_empty(&self) -> Result<bool> {
        self.with_read_lock(|| {
            self.store
                .is_empty()
                .map_err(|error| Error::Storage(error.to_string()))
        })
    }

    /// Streams quads matching a partial statement/context pattern.
    pub fn find(&self, pattern: StatementPattern<'_>) -> StatementMatches {
        self.with_read_lock(|| StatementMatches {
            inner: self.store.quads_for_pattern(
                pattern.subject,
                pattern.predicate,
                pattern.object,
                pattern.graph_name,
            ),
        })
    }

    /// Runs a store-mutating SPARQL Update under the write lock, then resyncs
    /// Fjall.
    pub(crate) fn run_sparql_update(
        &self,
        update: impl FnOnce(&Store) -> Result<()>,
    ) -> Result<()> {
        self.ensure_writable()?;
        let _guard = self
            .lock
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        update(&self.store)?;
        let Some(disk) = &self.disk else {
            return Ok(());
        };
        if let Err(error) = disk.replace_all_from_store(&self.store) {
            if let Err(reload_error) = self.reload_store_from_disk_unlocked(disk) {
                return Err(Error::Storage(format!(
                    "durable sync failed after SPARQL Update ({error}); rollback from disk also failed ({reload_error})"
                )));
            }
            return Err(Error::Storage(format!(
                "durable sync failed after SPARQL Update; in-memory store rolled back to disk: {error}"
            )));
        }
        Ok(())
    }

    fn reload_store_from_disk_unlocked(&self, disk: &DurableStore) -> Result<()> {
        self.store
            .clear()
            .map_err(|error| Error::Storage(error.to_string()))?;
        disk.load_into(&self.store)
    }
}
