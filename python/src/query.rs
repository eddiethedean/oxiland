use oxigraph::model::GraphName as OxGraphName;
use oxigraph::sparql::{QuerySolutionIter, QueryTripleIter};
use oxiland::{
    Model, Query, QueryResults, ResultsFormat, Update, serialize_query_results_to_string,
};
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PySequence, PyString};

use crate::error::map_error;
use crate::model::PyModel;
use crate::terms::{PyTriple, extract_graph_name, term_to_py, triple_to_py};

/// Owns a boxed model so Oxigraph iterators remain valid after transmute.
///
/// # Safety
/// The Oxigraph store handle inside [`Model`] is reference-counted. The boxed
/// model is allocated before `execute` and kept alive for the iterator
/// lifetime; the `'static` transmute is only sound while `_model` lives.
struct ModelGuard {
    _model: Box<Model>,
}

#[pyclass(name = "Solution", module = "oxiland", frozen)]
pub struct PySolution {
    ordered: Vec<(String, Option<oxigraph::model::Term>)>,
}

#[pymethods]
impl PySolution {
    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let Ok(index) = key.extract::<usize>() {
            return match self.ordered.get(index) {
                None => Err(pyo3::exceptions::PyKeyError::new_err(index)),
                Some((_, None)) => Ok(py.None()),
                Some((_, Some(term))) => term_to_py(py, term),
            };
        }
        if let Ok(name) = key.extract::<String>() {
            return match self.ordered.iter().find(|(n, _)| n == &name) {
                None => Err(pyo3::exceptions::PyKeyError::new_err(name)),
                Some((_, None)) => Ok(py.None()),
                Some((_, Some(term))) => term_to_py(py, term),
            };
        }
        Err(pyo3::exceptions::PyTypeError::new_err(
            "Solution key must be str or int",
        ))
    }

    fn get(&self, py: Python<'_>, name: &str) -> PyResult<Option<PyObject>> {
        match self.ordered.iter().find(|(n, _)| n == name) {
            None => Ok(None),
            Some((_, None)) => Ok(None),
            Some((_, Some(term))) => Ok(Some(term_to_py(py, term)?)),
        }
    }

    fn variables(&self) -> Vec<String> {
        self.ordered.iter().map(|(n, _)| n.clone()).collect()
    }

    fn __len__(&self) -> usize {
        self.ordered.len()
    }

    fn __repr__(&self) -> String {
        let names: Vec<&str> = self.ordered.iter().map(|(n, _)| n.as_str()).collect();
        format!("Solution({names:?})")
    }
}

enum SolutionsState {
    Live {
        _guard: ModelGuard,
        iter: QuerySolutionIter<'static>,
    },
    Done,
}

#[pyclass(name = "SolutionsIter", module = "oxiland", unsendable)]
pub struct PySolutionsIter {
    state: SolutionsState,
}

#[pymethods]
impl PySolutionsIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<PySolution>> {
        let SolutionsState::Live { iter, .. } = &mut self.state else {
            return Ok(None);
        };
        match iter.next() {
            None => {
                self.state = SolutionsState::Done;
                Ok(None)
            }
            Some(Ok(solution)) => {
                let variables = solution.variables();
                let mut ordered = Vec::with_capacity(variables.len());
                for (idx, variable) in variables.iter().enumerate() {
                    let name = variable.as_str().to_owned();
                    let term = solution.get(idx).cloned();
                    ordered.push((name, term));
                }
                Ok(Some(PySolution { ordered }))
            }
            Some(Err(error)) => Err(map_error(oxiland::Error::SparqlEvaluation(
                error.to_string(),
            ))),
        }
    }
}

enum TriplesState {
    Live {
        _guard: ModelGuard,
        iter: QueryTripleIter<'static>,
    },
    Done,
}

#[pyclass(name = "TriplesIter", module = "oxiland", unsendable)]
pub struct PyTriplesIter {
    state: TriplesState,
}

#[pymethods]
impl PyTriplesIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<PyTriple>> {
        let TriplesState::Live { iter, .. } = &mut self.state else {
            return Ok(None);
        };
        match iter.next() {
            None => {
                self.state = TriplesState::Done;
                Ok(None)
            }
            Some(Ok(triple)) => Ok(Some(triple_to_py(triple))),
            Some(Err(error)) => Err(map_error(oxiland::Error::SparqlEvaluation(
                error.to_string(),
            ))),
        }
    }
}

fn extract_graphs(graphs: &Bound<'_, PyAny>) -> PyResult<Vec<OxGraphName>> {
    if graphs.is_instance_of::<PyString>() || graphs.is_instance_of::<PyBytes>() {
        return Err(pyo3::exceptions::PyTypeError::new_err(
            "default_graph must be a graph name or a sequence of graph names",
        ));
    }
    if let Ok(graph) = extract_graph_name(graphs) {
        return Ok(vec![graph]);
    }
    if let Ok(seq) = graphs.downcast::<PySequence>() {
        let mut out = Vec::with_capacity(seq.len()?);
        for i in 0..seq.len()? {
            out.push(extract_graph_name(&seq.get_item(i)?)?);
        }
        return Ok(out);
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "default_graph must be a graph name or a sequence of graph names",
    ))
}

#[pyfunction]
#[pyo3(signature = (model, sparql, *, base_iri=None, limit=None, offset=None, default_graph=None, default_graph_as_union=false))]
#[allow(clippy::too_many_arguments)]
fn query(
    py: Python<'_>,
    model: &PyModel,
    sparql: &str,
    base_iri: Option<&str>,
    limit: Option<usize>,
    offset: Option<usize>,
    default_graph: Option<&Bound<'_, PyAny>>,
    default_graph_as_union: bool,
) -> PyResult<PyObject> {
    let mut q = Query::new(sparql);
    if let Some(base) = base_iri {
        q = q.base_iri(base).map_err(map_error)?;
    }
    if let Some(limit) = limit {
        q = q.limit(limit).map_err(map_error)?;
    }
    if let Some(offset) = offset {
        q = q.offset(offset).map_err(map_error)?;
    }
    if default_graph_as_union {
        q = q.default_graph_as_union();
    }
    if let Some(graphs) = default_graph {
        q = q.default_graph(extract_graphs(graphs)?);
    }

    // SAFETY: box the model before execute so the iterator's borrow target has
    // a stable address; keep the box alive in ModelGuard for the Python
    // iterator lifetime after transmute to `'static`.
    let guard = Box::new(model.inner.clone());
    let results = {
        let model_ptr: *const Model = guard.as_ref();
        unsafe { q.execute(&*model_ptr) }.map_err(map_error)?
    };

    match results {
        QueryResults::Boolean(value) => Ok(value.into_py_any(py)?),
        QueryResults::Solutions(iter) => {
            let iter: QuerySolutionIter<'static> = unsafe { std::mem::transmute(iter) };
            let obj = PySolutionsIter {
                state: SolutionsState::Live {
                    _guard: ModelGuard { _model: guard },
                    iter,
                },
            };
            Ok(obj.into_pyobject(py)?.into_any().unbind())
        }
        QueryResults::Graph(iter) => {
            let iter: QueryTripleIter<'static> = unsafe { std::mem::transmute(iter) };
            let obj = PyTriplesIter {
                state: TriplesState::Live {
                    _guard: ModelGuard { _model: guard },
                    iter,
                },
            };
            Ok(obj.into_pyobject(py)?.into_any().unbind())
        }
    }
}

#[pyfunction]
#[pyo3(signature = (model, sparql, *, base_iri=None, default_graph=None, default_graph_as_union=false))]
fn update(
    model: &PyModel,
    sparql: &str,
    base_iri: Option<&str>,
    default_graph: Option<&Bound<'_, PyAny>>,
    default_graph_as_union: bool,
) -> PyResult<()> {
    let mut u = Update::new(sparql);
    if let Some(base) = base_iri {
        u = u.base_iri(base).map_err(map_error)?;
    }
    if default_graph_as_union {
        u = u.default_graph_as_union();
    }
    if let Some(graphs) = default_graph {
        u = u.default_graph(extract_graphs(graphs)?);
    }
    u.execute(&model.inner).map_err(map_error)
}

#[pyfunction]
#[pyo3(signature = (model, sparql, format, *, base_iri=None, limit=None, offset=None))]
fn serialize_results(
    model: &PyModel,
    sparql: &str,
    format: &str,
    base_iri: Option<&str>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> PyResult<String> {
    let mut q = Query::new(sparql);
    if let Some(base) = base_iri {
        q = q.base_iri(base).map_err(map_error)?;
    }
    if let Some(limit) = limit {
        q = q.limit(limit).map_err(map_error)?;
    }
    if let Some(offset) = offset {
        q = q.offset(offset).map_err(map_error)?;
    }
    let results = q.execute(&model.inner).map_err(map_error)?;
    let format = ResultsFormat::from_name(format).map_err(map_error)?;
    serialize_query_results_to_string(results, format).map_err(map_error)
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySolution>()?;
    m.add_class::<PySolutionsIter>()?;
    m.add_class::<PyTriplesIter>()?;
    m.add_function(wrap_pyfunction!(query, m)?)?;
    m.add_function(wrap_pyfunction!(update, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_results, m)?)?;
    Ok(())
}
