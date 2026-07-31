use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use oxigraph::model::{GraphName as OxGraphName, NamedNodeRef, NamedOrBlankNodeRef, Quad, TermRef};
use oxiland::storage::{compiled_backends, OpenOptions};
use oxiland::{Model, StatementMatches, StatementPattern, StorageBackend};
use pyo3::prelude::*;

use crate::error::{map_error, path_buf};
use crate::terms::{
    PyNamedNode, PyQuad, extract_graph_name, extract_named_or_blank, extract_term, extract_triple,
    quad_to_py,
};

#[pyclass(name = "Model", module = "oxiland")]
pub struct PyModel {
    pub(crate) inner: Model,
    transaction_active: Arc<AtomicBool>,
}

#[pymethods]
impl PyModel {
    #[new]
    fn new() -> PyResult<Self> {
        Ok(Self {
            inner: Model::new().map_err(map_error)?,
            transaction_active: Arc::new(AtomicBool::new(false)),
        })
    }

    #[classmethod]
    #[pyo3(signature = (path, *, read_only=false, create=true, backend="fjall"))]
    fn open(
        _cls: &Bound<'_, pyo3::types::PyType>,
        path: &Bound<'_, PyAny>,
        read_only: bool,
        create: bool,
        backend: &str,
    ) -> PyResult<Self> {
        let path = path_buf(path)?;
        let backend = StorageBackend::from_name(backend).map_err(map_error)?;
        let options = OpenOptions::new(backend, path)
            .read_only(read_only)
            .create(create);
        Ok(Self {
            inner: Model::open_with(options).map_err(map_error)?,
            transaction_active: Arc::new(AtomicBool::new(false)),
        })
    }

    #[classmethod]
    fn migrate_legacy_store(
        _cls: &Bound<'_, pyo3::types::PyType>,
        path: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: Model::migrate_legacy_store(path_buf(path)?).map_err(map_error)?,
            transaction_active: Arc::new(AtomicBool::new(false)),
        })
    }

    #[getter]
    fn backend(&self) -> &'static str {
        match self.inner.backend() {
            StorageBackend::Memory => "memory",
            StorageBackend::Fjall => "fjall",
        }
    }

    #[pyo3(signature = (statement, graph=None))]
    fn add(
        &self,
        statement: &Bound<'_, PyAny>,
        graph: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        let triple = extract_triple(statement)?;
        match graph {
            None => self.inner.add(triple).map_err(map_error),
            Some(g) => self
                .inner
                .add_to_graph(triple, extract_graph_name(g)?)
                .map_err(map_error),
        }
    }

    fn insert_quad(&self, quad: &PyQuad) -> PyResult<bool> {
        self.inner
            .insert_quad(quad.inner.clone())
            .map_err(map_error)
    }

    #[pyo3(signature = (statement, graph=None))]
    fn remove(
        &self,
        statement: &Bound<'_, PyAny>,
        graph: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        let triple = extract_triple(statement)?;
        match graph {
            None => self.inner.remove(triple).map_err(map_error),
            Some(g) => self
                .inner
                .remove_from_graph(triple, extract_graph_name(g)?)
                .map_err(map_error),
        }
    }

    fn remove_quad(&self, quad: &PyQuad) -> PyResult<bool> {
        self.inner.remove_quad(&quad.inner).map_err(map_error)
    }

    #[pyo3(signature = (statement, graph=None))]
    fn contains(
        &self,
        statement: &Bound<'_, PyAny>,
        graph: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        let triple = extract_triple(statement)?;
        match graph {
            None => self.inner.contains(triple.as_ref()).map_err(map_error),
            Some(g) => self
                .inner
                .contains_in_graph(triple.as_ref(), extract_graph_name(g)?.as_ref())
                .map_err(map_error),
        }
    }

    fn clear(&self) -> PyResult<()> {
        self.inner.clear().map_err(map_error)
    }

    fn clear_graph(&self, graph: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner
            .clear_graph(extract_graph_name(graph)?)
            .map_err(map_error)
    }

    fn sync(&self) -> PyResult<()> {
        self.inner.sync().map_err(map_error)
    }

    fn __len__(&self) -> PyResult<usize> {
        self.inner.len().map_err(map_error)
    }

    fn is_empty(&self) -> PyResult<bool> {
        self.inner.is_empty().map_err(map_error)
    }

    fn export_nquads(&self, path: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner
            .export_nquads_to_path(path_buf(path)?)
            .map_err(map_error)
    }

    fn import_nquads(&self, path: &Bound<'_, PyAny>) -> PyResult<usize> {
        self.inner
            .import_nquads_from_path(path_buf(path)?)
            .map_err(map_error)
    }

    #[pyo3(signature = (*, subject=None, predicate=None, object=None, graph=None))]
    fn find(
        &self,
        subject: Option<&Bound<'_, PyAny>>,
        predicate: Option<&PyNamedNode>,
        object: Option<&Bound<'_, PyAny>>,
        graph: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<PyFindIter> {
        // Hold owned terms so refs in StatementPattern remain valid for the call.
        let subject_owned = match subject {
            Some(s) => Some(extract_named_or_blank(s)?),
            None => None,
        };
        let object_owned = match object {
            Some(o) => Some(extract_term(o)?),
            None => None,
        };
        let graph_owned = match graph {
            Some(g) => Some(extract_graph_name(g)?),
            None => None,
        };

        let subject_ref = subject_owned.as_ref().map(|s| match s {
            oxigraph::model::NamedOrBlankNode::NamedNode(n) => {
                NamedOrBlankNodeRef::NamedNode(n.as_ref())
            }
            oxigraph::model::NamedOrBlankNode::BlankNode(n) => {
                NamedOrBlankNodeRef::BlankNode(n.as_ref())
            }
        });
        let predicate_ref = predicate.map(|p| NamedNodeRef::new_unchecked(p.inner.as_str()));
        let object_ref = object_owned.as_ref().map(|t| match t {
            oxigraph::model::Term::NamedNode(n) => TermRef::NamedNode(n.as_ref()),
            oxigraph::model::Term::BlankNode(n) => TermRef::BlankNode(n.as_ref()),
            oxigraph::model::Term::Literal(l) => TermRef::Literal(l.as_ref()),
        });
        let graph_ref = graph_owned.as_ref().map(|g| g.as_ref());

        let matches = self.inner.find(StatementPattern {
            subject: subject_ref,
            predicate: predicate_ref,
            object: object_ref,
            graph_name: graph_ref,
        });
        Ok(PyFindIter { inner: matches })
    }

    fn transaction(&self) -> PyTransaction {
        PyTransaction {
            model: self.inner.clone(),
            ops: Mutex::new(Vec::new()),
            transaction_active: Arc::clone(&self.transaction_active),
            entered: false,
        }
    }

    fn __repr__(&self) -> String {
        let backend = match self.inner.backend() {
            StorageBackend::Memory => "memory",
            StorageBackend::Fjall => "fjall",
        };
        format!("Model(backend={backend:?})")
    }
}

#[pyclass(name = "FindIter", module = "oxiland")]
pub struct PyFindIter {
    inner: StatementMatches,
}

#[pymethods]
impl PyFindIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<PyQuad>> {
        match self.inner.next() {
            None => Ok(None),
            Some(Ok(quad)) => Ok(Some(quad_to_py(quad))),
            Some(Err(error)) => Err(map_error(error)),
        }
    }
}

#[derive(Clone)]
enum TxnOp {
    Add(Quad),
    Remove(Quad),
    Clear,
    ClearGraph(OxGraphName),
}

#[pyclass(name = "Transaction", module = "oxiland")]
pub struct PyTransaction {
    model: Model,
    ops: Mutex<Vec<TxnOp>>,
    transaction_active: Arc<AtomicBool>,
    entered: bool,
}

#[pymethods]
impl PyTransaction {
    fn __enter__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRefMut<'_, Self>> {
        if slf.entered
            || slf
                .transaction_active
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
        {
            return Err(map_error(oxiland::Error::Unsupported(
                "nested transaction is unsupported".into(),
            )));
        }
        slf.entered = true;
        Ok(slf)
    }

    fn __exit__(
        &mut self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        self.ensure_entered()?;
        if exc_value.is_some() {
            self.ops
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clear();
            self.finish();
            return Ok(false);
        }
        let ops = std::mem::take(
            &mut *self
                .ops
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        let result = self.model.transaction(|txn| {
            for op in ops {
                match op {
                    TxnOp::Add(quad) => {
                        txn.insert_quad(quad)?;
                    }
                    TxnOp::Remove(quad) => {
                        txn.remove_quad(&quad)?;
                    }
                    TxnOp::Clear => txn.clear()?,
                    TxnOp::ClearGraph(g) => txn.clear_graph(g)?,
                }
            }
            Ok(())
        });
        self.finish();
        result.map_err(map_error)?;
        Ok(false)
    }

    #[pyo3(signature = (statement, graph=None))]
    fn add(&self, statement: &Bound<'_, PyAny>, graph: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        self.ensure_entered()?;
        let triple = extract_triple(statement)?;
        let graph_name = match graph {
            None => OxGraphName::DefaultGraph,
            Some(g) => extract_graph_name(g)?,
        };
        let quad = Quad::new(triple.subject, triple.predicate, triple.object, graph_name);
        self.ops
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(TxnOp::Add(quad));
        Ok(())
    }

    fn insert_quad(&self, quad: &PyQuad) -> PyResult<()> {
        self.ensure_entered()?;
        self.ops
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(TxnOp::Add(quad.inner.clone()));
        Ok(())
    }

    #[pyo3(signature = (statement, graph=None))]
    fn remove(
        &self,
        statement: &Bound<'_, PyAny>,
        graph: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        self.ensure_entered()?;
        let triple = extract_triple(statement)?;
        let graph_name = match graph {
            None => OxGraphName::DefaultGraph,
            Some(g) => extract_graph_name(g)?,
        };
        let quad = Quad::new(triple.subject, triple.predicate, triple.object, graph_name);
        self.ops
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(TxnOp::Remove(quad));
        Ok(())
    }

    fn remove_quad(&self, quad: &PyQuad) -> PyResult<()> {
        self.ensure_entered()?;
        self.ops
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(TxnOp::Remove(quad.inner.clone()));
        Ok(())
    }

    fn clear(&self) -> PyResult<()> {
        self.ensure_entered()?;
        self.ops
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(TxnOp::Clear);
        Ok(())
    }

    fn clear_graph(&self, graph: &Bound<'_, PyAny>) -> PyResult<()> {
        self.ensure_entered()?;
        self.ops
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(TxnOp::ClearGraph(extract_graph_name(graph)?));
        Ok(())
    }
}

impl PyTransaction {
    fn ensure_entered(&self) -> PyResult<()> {
        if !self.entered {
            return Err(map_error(oxiland::Error::Unsupported(
                "transaction methods require an active `with model.transaction()` block".into(),
            )));
        }
        Ok(())
    }

    fn finish(&mut self) {
        self.entered = false;
        self.transaction_active.store(false, Ordering::Release);
    }
}

impl Drop for PyTransaction {
    fn drop(&mut self) {
        if self.entered {
            self.transaction_active.store(false, Ordering::Release);
        }
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyModel>()?;
    m.add_class::<PyFindIter>()?;
    m.add_class::<PyTransaction>()?;
    m.add_function(wrap_pyfunction!(py_compiled_backends, m)?)?;
    m.add_function(wrap_pyfunction!(py_storage_backend_available, m)?)?;
    Ok(())
}

#[pyfunction(name = "compiled_backends")]
fn py_compiled_backends() -> Vec<&'static str> {
    compiled_backends().iter().map(|b| b.name()).collect()
}

#[pyfunction(name = "storage_backend_available")]
fn py_storage_backend_available(name: &str) -> PyResult<bool> {
    Model::storage_backend_available(name).map_err(map_error)
}
