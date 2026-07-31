use oxigraph::model::{
    BlankNode as OxBlank, GraphName as OxGraphName, Literal as OxLiteral, NamedNode as OxNamed,
    NamedOrBlankNode, Quad as OxQuad, Term as OxTerm, Triple as OxTriple,
};
use oxiland::terms;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;

use crate::error::map_error;

#[pyclass(name = "NamedNode", module = "oxiland", frozen, skip_from_py_object)]
#[derive(Clone, Debug)]
pub struct PyNamedNode {
    pub(crate) inner: OxNamed,
}

#[pymethods]
impl PyNamedNode {
    #[new]
    fn new(iri: &str) -> PyResult<Self> {
        Ok(Self {
            inner: terms::named_node(iri).map_err(map_error)?,
        })
    }

    #[getter]
    fn value(&self) -> &str {
        self.inner.as_str()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("NamedNode({:?})", self.inner.as_str())
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.inner.as_str().hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "NamedNode only supports == and !=",
            )),
        }
    }
}

#[pyclass(name = "BlankNode", module = "oxiland", frozen, skip_from_py_object)]
#[derive(Clone, Debug)]
pub struct PyBlankNode {
    pub(crate) inner: OxBlank,
}

#[pymethods]
impl PyBlankNode {
    #[new]
    #[pyo3(signature = (id=None))]
    fn new(id: Option<&str>) -> PyResult<Self> {
        Ok(Self {
            inner: terms::blank_node(id).map_err(map_error)?,
        })
    }

    #[getter]
    fn value(&self) -> String {
        self.inner.as_str().to_owned()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("BlankNode({:?})", self.inner.as_str())
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.inner.as_str().hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "BlankNode only supports == and !=",
            )),
        }
    }
}

#[pyclass(name = "Literal", module = "oxiland", frozen, skip_from_py_object)]
#[derive(Clone, Debug)]
pub struct PyLiteral {
    pub(crate) inner: OxLiteral,
}

#[pymethods]
impl PyLiteral {
    #[new]
    #[pyo3(signature = (value, *, language=None, datatype=None))]
    fn new(value: &str, language: Option<&str>, datatype: Option<&PyNamedNode>) -> PyResult<Self> {
        let inner = match (language, datatype) {
            (Some(language), None) => OxLiteral::new_language_tagged_literal(value, language)
                .map_err(|e| map_error(oxiland::Error::InvalidRdf(e.to_string())))?,
            (None, Some(datatype)) => OxLiteral::new_typed_literal(value, datatype.inner.clone()),
            (None, None) => OxLiteral::new_simple_literal(value),
            (Some(_), Some(_)) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "Literal cannot have both language and datatype",
                ));
            }
        };
        Ok(Self { inner })
    }

    #[getter]
    fn value(&self) -> &str {
        self.inner.value()
    }

    #[getter]
    fn language(&self) -> Option<&str> {
        self.inner.language()
    }

    #[getter]
    fn datatype(&self) -> PyNamedNode {
        PyNamedNode {
            inner: self.inner.datatype().into_owned(),
        }
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Literal({:?})", self.inner.value())
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.inner.to_string().hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "Literal only supports == and !=",
            )),
        }
    }
}

#[pyclass(name = "Triple", module = "oxiland", frozen, skip_from_py_object)]
#[derive(Clone, Debug)]
pub struct PyTriple {
    pub(crate) inner: OxTriple,
}

#[pymethods]
impl PyTriple {
    #[new]
    fn new(
        subject: &Bound<'_, PyAny>,
        predicate: &PyNamedNode,
        object: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: OxTriple::new(
                extract_named_or_blank(subject)?,
                predicate.inner.clone(),
                extract_term(object)?,
            ),
        })
    }

    #[getter]
    fn subject(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        named_or_blank_to_py(py, &self.inner.subject)
    }

    #[getter]
    fn predicate(&self) -> PyNamedNode {
        PyNamedNode {
            inner: self.inner.predicate.clone(),
        }
    }

    #[getter]
    fn object(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        term_to_py(py, &self.inner.object)
    }

    fn __repr__(&self) -> String {
        format!("Triple({:?})", self.inner)
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.inner.hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "Triple only supports == and !=",
            )),
        }
    }
}

#[pyclass(name = "Quad", module = "oxiland", frozen, skip_from_py_object)]
#[derive(Clone, Debug)]
pub struct PyQuad {
    pub(crate) inner: OxQuad,
}

#[pymethods]
impl PyQuad {
    #[new]
    #[pyo3(signature = (subject, predicate, object, graph=None))]
    fn new(
        subject: &Bound<'_, PyAny>,
        predicate: &PyNamedNode,
        object: &Bound<'_, PyAny>,
        graph: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let graph_name = match graph {
            None => OxGraphName::DefaultGraph,
            Some(g) => extract_graph_name(g)?,
        };
        Ok(Self {
            inner: OxQuad::new(
                extract_named_or_blank(subject)?,
                predicate.inner.clone(),
                extract_term(object)?,
                graph_name,
            ),
        })
    }

    #[getter]
    fn subject(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        named_or_blank_to_py(py, &self.inner.subject)
    }

    #[getter]
    fn predicate(&self) -> PyNamedNode {
        PyNamedNode {
            inner: self.inner.predicate.clone(),
        }
    }

    #[getter]
    fn object(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        term_to_py(py, &self.inner.object)
    }

    #[getter]
    fn graph(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        graph_name_to_py(py, &self.inner.graph_name)
    }

    fn __repr__(&self) -> String {
        format!("Quad({:?})", self.inner)
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.inner.hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "Quad only supports == and !=",
            )),
        }
    }
}

#[pyclass(name = "DefaultGraph", module = "oxiland", frozen, skip_from_py_object)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PyDefaultGraph;

#[pymethods]
impl PyDefaultGraph {
    #[new]
    fn new() -> Self {
        Self
    }

    fn __repr__(&self) -> &'static str {
        "DefaultGraph()"
    }

    fn __richcmp__(&self, _other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(true),
            CompareOp::Ne => Ok(false),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "DefaultGraph only supports == and !=",
            )),
        }
    }
}

#[pyfunction]
fn named_node(iri: &str) -> PyResult<PyNamedNode> {
    PyNamedNode::new(iri)
}

#[pyfunction]
#[pyo3(signature = (id=None))]
fn blank_node(id: Option<&str>) -> PyResult<PyBlankNode> {
    PyBlankNode::new(id)
}

pub fn extract_term(obj: &Bound<'_, PyAny>) -> PyResult<OxTerm> {
    if let Ok(node) = obj.extract::<PyRef<'_, PyNamedNode>>() {
        return Ok(OxTerm::NamedNode(node.inner.clone()));
    }
    if let Ok(node) = obj.extract::<PyRef<'_, PyBlankNode>>() {
        return Ok(OxTerm::BlankNode(node.inner.clone()));
    }
    if let Ok(lit) = obj.extract::<PyRef<'_, PyLiteral>>() {
        return Ok(OxTerm::Literal(lit.inner.clone()));
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "expected NamedNode, BlankNode, or Literal",
    ))
}

pub fn extract_named_or_blank(obj: &Bound<'_, PyAny>) -> PyResult<NamedOrBlankNode> {
    if let Ok(node) = obj.extract::<PyRef<'_, PyNamedNode>>() {
        return Ok(NamedOrBlankNode::NamedNode(node.inner.clone()));
    }
    if let Ok(node) = obj.extract::<PyRef<'_, PyBlankNode>>() {
        return Ok(NamedOrBlankNode::BlankNode(node.inner.clone()));
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "expected NamedNode or BlankNode",
    ))
}

pub fn extract_graph_name(obj: &Bound<'_, PyAny>) -> PyResult<OxGraphName> {
    if obj.is_none() {
        return Ok(OxGraphName::DefaultGraph);
    }
    if obj.extract::<PyRef<'_, PyDefaultGraph>>().is_ok() {
        return Ok(OxGraphName::DefaultGraph);
    }
    if let Ok(node) = obj.extract::<PyRef<'_, PyNamedNode>>() {
        return Ok(OxGraphName::NamedNode(node.inner.clone()));
    }
    if let Ok(node) = obj.extract::<PyRef<'_, PyBlankNode>>() {
        return Ok(OxGraphName::BlankNode(node.inner.clone()));
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "expected DefaultGraph, NamedNode, BlankNode, or None",
    ))
}

pub fn extract_triple(obj: &Bound<'_, PyAny>) -> PyResult<OxTriple> {
    if let Ok(triple) = obj.extract::<PyRef<'_, PyTriple>>() {
        return Ok(triple.inner.clone());
    }
    Err(pyo3::exceptions::PyTypeError::new_err("expected Triple"))
}

pub fn term_to_py(py: Python<'_>, term: &OxTerm) -> PyResult<Py<PyAny>> {
    match term {
        OxTerm::NamedNode(n) => Ok(PyNamedNode { inner: n.clone() }
            .into_pyobject(py)?
            .into_any()
            .unbind()),
        OxTerm::BlankNode(n) => Ok(PyBlankNode { inner: n.clone() }
            .into_pyobject(py)?
            .into_any()
            .unbind()),
        OxTerm::Literal(l) => Ok(PyLiteral { inner: l.clone() }
            .into_pyobject(py)?
            .into_any()
            .unbind()),
    }
}

pub fn named_or_blank_to_py(py: Python<'_>, node: &NamedOrBlankNode) -> PyResult<Py<PyAny>> {
    match node {
        NamedOrBlankNode::NamedNode(n) => Ok(PyNamedNode { inner: n.clone() }
            .into_pyobject(py)?
            .into_any()
            .unbind()),
        NamedOrBlankNode::BlankNode(n) => Ok(PyBlankNode { inner: n.clone() }
            .into_pyobject(py)?
            .into_any()
            .unbind()),
    }
}

pub fn graph_name_to_py(py: Python<'_>, graph: &OxGraphName) -> PyResult<Py<PyAny>> {
    match graph {
        OxGraphName::DefaultGraph => Ok(PyDefaultGraph.into_pyobject(py)?.into_any().unbind()),
        OxGraphName::NamedNode(n) => Ok(PyNamedNode { inner: n.clone() }
            .into_pyobject(py)?
            .into_any()
            .unbind()),
        OxGraphName::BlankNode(n) => Ok(PyBlankNode { inner: n.clone() }
            .into_pyobject(py)?
            .into_any()
            .unbind()),
    }
}

pub fn quad_to_py(quad: OxQuad) -> PyQuad {
    PyQuad { inner: quad }
}

pub fn triple_to_py(triple: OxTriple) -> PyTriple {
    PyTriple { inner: triple }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyNamedNode>()?;
    m.add_class::<PyBlankNode>()?;
    m.add_class::<PyLiteral>()?;
    m.add_class::<PyTriple>()?;
    m.add_class::<PyQuad>()?;
    m.add_class::<PyDefaultGraph>()?;
    m.add_function(wrap_pyfunction!(named_node, m)?)?;
    m.add_function(wrap_pyfunction!(blank_node, m)?)?;
    Ok(())
}
