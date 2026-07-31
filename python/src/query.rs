use std::collections::HashMap;

use oxigraph::sparql::{QuerySolutionIter, QueryTripleIter};
use oxiland::{
    Model, Query, QueryResults, ResultsFormat, Update, serialize_query_results_to_string,
};
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::error::map_error;
use crate::model::PyModel;
use crate::terms::{PyTriple, extract_graph_name, term_to_py, triple_to_py};

/// Holds a model so Oxigraph iterators remain valid (store handle is shared).
struct ModelGuard {
    _model: Model,
}

#[pyclass(name = "Solution", module = "oxiland", frozen)]
pub struct PySolution {
    values: HashMap<String, oxigraph::model::Term>,
    ordered: Vec<(String, Option<oxigraph::model::Term>)>,
}

#[pymethods]
impl PySolution {
    fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let Ok(index) = key.extract::<usize>() {
            let term = self
                .ordered
                .get(index)
                .and_then(|(_, t)| t.as_ref())
                .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(index))?;
            return term_to_py(py, term);
        }
        if let Ok(name) = key.extract::<String>() {
            let term = self
                .values
                .get(&name)
                .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(name))?;
            return term_to_py(py, term);
        }
        Err(pyo3::exceptions::PyTypeError::new_err(
            "Solution key must be str or int",
        ))
    }

    fn get(&self, py: Python<'_>, name: &str) -> PyResult<Option<PyObject>> {
        match self.values.get(name) {
            Some(term) => Ok(Some(term_to_py(py, term)?)),
            None => Ok(None),
        }
    }

    fn variables(&self) -> Vec<String> {
        self.ordered.iter().map(|(n, _)| n.clone()).collect()
    }

    fn __len__(&self) -> usize {
        self.ordered.len()
    }

    fn __repr__(&self) -> String {
        format!("Solution({:?})", self.values.keys().collect::<Vec<_>>())
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
                let mut values = HashMap::new();
                let mut ordered = Vec::with_capacity(variables.len());
                for (idx, variable) in variables.iter().enumerate() {
                    let name = variable.as_str().to_owned();
                    let term = solution.get(idx).cloned();
                    if let Some(ref t) = term {
                        values.insert(name.clone(), t.clone());
                    }
                    ordered.push((name, term));
                }
                Ok(Some(PySolution { values, ordered }))
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
        let list = if let Ok(seq) = graphs.downcast::<PyList>() {
            let mut out = Vec::new();
            for item in seq.iter() {
                out.push(extract_graph_name(&item)?);
            }
            out
        } else {
            vec![extract_graph_name(graphs)?]
        };
        q = q.default_graph(list);
    }

    // SAFETY: `guard` owns a Model clone that shares the store handle. The
    // Oxigraph iterators borrow that store; we keep `guard` alive in the
    // Python iterator and transmute the lifetime to `'static`.
    let guard_model = model.inner.clone();
    let model_ptr: *const Model = &guard_model;
    let results = unsafe { q.execute(&*model_ptr) }.map_err(map_error)?;

    match results {
        QueryResults::Boolean(value) => Ok(value.into_py_any(py)?),
        QueryResults::Solutions(iter) => {
            let iter: QuerySolutionIter<'static> = unsafe { std::mem::transmute(iter) };
            let obj = PySolutionsIter {
                state: SolutionsState::Live {
                    _guard: ModelGuard {
                        _model: guard_model,
                    },
                    iter,
                },
            };
            Ok(obj.into_pyobject(py)?.into_any().unbind())
        }
        QueryResults::Graph(iter) => {
            let iter: QueryTripleIter<'static> = unsafe { std::mem::transmute(iter) };
            let obj = PyTriplesIter {
                state: TriplesState::Live {
                    _guard: ModelGuard {
                        _model: guard_model,
                    },
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
        let list = if let Ok(seq) = graphs.downcast::<PyList>() {
            let mut out = Vec::new();
            for item in seq.iter() {
                out.push(extract_graph_name(&item)?);
            }
            out
        } else {
            vec![extract_graph_name(graphs)?]
        };
        u = u.default_graph(list);
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
