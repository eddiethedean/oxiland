use std::fs::File;
use std::io::{BufReader, Cursor};

use oxigraph::model::Quad;
use oxiland::io::{BomStrippingReader, GraphTarget, Parser, QuadStream, Serializer, Syntax};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::error::{map_error, path_buf};
use crate::model::PyModel;
use crate::terms::{PyQuad, extract_graph_name, quad_to_py};

#[pyclass(name = "Syntax", module = "oxiland", frozen, skip_from_py_object)]
#[derive(Clone, Copy, Debug)]
pub struct PySyntax {
    pub(crate) inner: Syntax,
}

#[pymethods]
impl PySyntax {
    #[classattr]
    const TURTLE: Self = Self {
        inner: Syntax::Turtle,
    };
    #[classattr]
    const NTRIPLES: Self = Self {
        inner: Syntax::NTriples,
    };
    #[classattr]
    const NQUADS: Self = Self {
        inner: Syntax::NQuads,
    };
    #[classattr]
    const TRIG: Self = Self {
        inner: Syntax::TriG,
    };
    #[classattr]
    const RDFXML: Self = Self {
        inner: Syntax::RdfXml,
    };

    #[classmethod]
    fn from_name(_cls: &Bound<'_, pyo3::types::PyType>, name: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Syntax::from_name(name).map_err(map_error)?,
        })
    }

    #[classmethod]
    fn from_media_type(_cls: &Bound<'_, pyo3::types::PyType>, media_type: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Syntax::from_media_type(media_type).map_err(map_error)?,
        })
    }

    #[classmethod]
    fn from_extension(_cls: &Bound<'_, pyo3::types::PyType>, extension: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Syntax::from_extension(extension).map_err(map_error)?,
        })
    }

    #[getter]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    #[getter]
    fn media_type(&self) -> &'static str {
        self.inner.media_type()
    }

    #[getter]
    fn extension(&self) -> &'static str {
        self.inner.extension()
    }

    fn __repr__(&self) -> String {
        format!("Syntax({:?})", self.inner.name())
    }
}

fn resolve_syntax(syntax: &Bound<'_, PyAny>) -> PyResult<Syntax> {
    if let Ok(s) = syntax.extract::<PyRef<'_, PySyntax>>() {
        return Ok(s.inner);
    }
    if let Ok(name) = syntax.extract::<String>() {
        return Syntax::from_name(&name).map_err(map_error);
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "syntax must be Syntax or str",
    ))
}

fn build_parser(
    syntax: Syntax,
    base_iri: Option<&str>,
    graph: Option<&Bound<'_, PyAny>>,
) -> PyResult<Parser> {
    let mut parser = Parser::for_syntax(syntax);
    if let Some(base) = base_iri {
        parser = parser.base_iri(base).map_err(map_error)?;
    }
    if let Some(g) = graph {
        let target = match extract_graph_name(g)? {
            oxigraph::model::GraphName::DefaultGraph => GraphTarget::DefaultGraph,
            other => GraphTarget::Named(other),
        };
        parser = parser.graph_target(target);
    }
    Ok(parser)
}

enum OwnedStream {
    Cursor(QuadStream<BomStrippingReader<Cursor<Vec<u8>>>>),
    File(QuadStream<BomStrippingReader<BufReader<File>>>),
}

impl Iterator for OwnedStream {
    type Item = oxiland::Result<Quad>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Cursor(s) => s.next(),
            Self::File(s) => s.next(),
        }
    }
}

#[pyclass(name = "ParseIter", module = "oxiland")]
pub struct PyParseIter {
    inner: OwnedStream,
}

#[pymethods]
impl PyParseIter {
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

#[pyfunction]
#[pyo3(signature = (data, syntax, *, base_iri=None, graph=None))]
fn parse(
    data: &Bound<'_, PyAny>,
    syntax: &Bound<'_, PyAny>,
    base_iri: Option<&str>,
    graph: Option<&Bound<'_, PyAny>>,
) -> PyResult<PyParseIter> {
    let syntax = resolve_syntax(syntax)?;
    let parser = build_parser(syntax, base_iri, graph)?;
    let bytes = extract_bytes(data)?;
    let stream = parser.parse_reader(Cursor::new(bytes)).map_err(map_error)?;
    Ok(PyParseIter {
        inner: OwnedStream::Cursor(stream),
    })
}

#[pyfunction]
#[pyo3(signature = (path, syntax=None, *, base_iri=None, graph=None))]
fn parse_path(
    path: &Bound<'_, PyAny>,
    syntax: Option<&Bound<'_, PyAny>>,
    base_iri: Option<&str>,
    graph: Option<&Bound<'_, PyAny>>,
) -> PyResult<PyParseIter> {
    let path = path_buf(path)?;
    let syntax = match syntax {
        Some(s) => resolve_syntax(s)?,
        None => {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default();
            Syntax::from_extension(ext).map_err(map_error)?
        }
    };
    let parser = build_parser(syntax, base_iri, graph)?;
    let stream = parser.parse_path(&path).map_err(map_error)?;
    Ok(PyParseIter {
        inner: OwnedStream::File(stream),
    })
}

#[pyfunction]
#[pyo3(signature = (model, data, syntax, *, collecting=true, transactional=false, base_iri=None, graph=None))]
fn load(
    model: &PyModel,
    data: &Bound<'_, PyAny>,
    syntax: &Bound<'_, PyAny>,
    collecting: bool,
    transactional: bool,
    base_iri: Option<&str>,
    graph: Option<&Bound<'_, PyAny>>,
) -> PyResult<usize> {
    let syntax = resolve_syntax(syntax)?;
    let parser = build_parser(syntax, base_iri, graph)?;
    let bytes = extract_bytes(data)?;
    let cursor = Cursor::new(bytes);
    if transactional {
        parser
            .load_transactional(&model.inner, cursor)
            .map_err(map_error)
    } else if collecting {
        parser
            .load_collecting(&model.inner, cursor)
            .map_err(map_error)
    } else {
        parser.load_into(&model.inner, cursor).map_err(map_error)
    }
}

#[pyfunction]
#[pyo3(signature = (model, path, syntax=None, *, collecting=true, transactional=false, base_iri=None, graph=None))]
fn load_path(
    model: &PyModel,
    path: &Bound<'_, PyAny>,
    syntax: Option<&Bound<'_, PyAny>>,
    collecting: bool,
    transactional: bool,
    base_iri: Option<&str>,
    graph: Option<&Bound<'_, PyAny>>,
) -> PyResult<usize> {
    let path = path_buf(path)?;
    let syntax = match syntax {
        Some(s) => resolve_syntax(s)?,
        None => {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default();
            Syntax::from_extension(ext).map_err(map_error)?
        }
    };
    let parser = build_parser(syntax, base_iri, graph)?;
    if transactional {
        parser
            .load_path_transactional(&model.inner, &path)
            .map_err(map_error)
    } else if collecting {
        parser
            .load_path_collecting(&model.inner, &path)
            .map_err(map_error)
    } else {
        parser
            .load_path_into(&model.inner, &path)
            .map_err(map_error)
    }
}

#[pyfunction]
#[pyo3(signature = (model, syntax, *, base_iri=None, prefixes=None))]
fn serialize(
    model: &PyModel,
    syntax: &Bound<'_, PyAny>,
    base_iri: Option<&str>,
    prefixes: Option<std::collections::HashMap<String, String>>,
) -> PyResult<String> {
    let mut serializer = Serializer::for_syntax(resolve_syntax(syntax)?);
    if let Some(base) = base_iri {
        serializer = serializer.base_iri(base).map_err(map_error)?;
    }
    if let Some(prefixes) = prefixes {
        for (prefix, iri) in prefixes {
            serializer = serializer.with_prefix(prefix, iri).map_err(map_error)?;
        }
    }
    serializer
        .serialize_model_to_string(&model.inner)
        .map_err(map_error)
}

#[pyfunction]
#[pyo3(signature = (model, path, syntax=None, *, base_iri=None))]
fn serialize_path(
    model: &PyModel,
    path: &Bound<'_, PyAny>,
    syntax: Option<&Bound<'_, PyAny>>,
    base_iri: Option<&str>,
) -> PyResult<()> {
    let path = path_buf(path)?;
    let syntax = match syntax {
        Some(s) => resolve_syntax(s)?,
        None => {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default();
            Syntax::from_extension(ext).map_err(map_error)?
        }
    };
    let mut serializer = Serializer::for_syntax(syntax);
    if let Some(base) = base_iri {
        serializer = serializer.base_iri(base).map_err(map_error)?;
    }
    serializer
        .serialize_model_to_path(&model.inner, &path)
        .map_err(map_error)
}

fn extract_bytes(data: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    if let Ok(s) = data.extract::<String>() {
        return Ok(s.into_bytes());
    }
    if let Ok(b) = data.cast::<PyBytes>() {
        return Ok(b.as_bytes().to_vec());
    }
    if let Ok(b) = data.extract::<Vec<u8>>() {
        return Ok(b);
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "data must be str or bytes",
    ))
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySyntax>()?;
    m.add_class::<PyParseIter>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_path, m)?)?;
    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_function(wrap_pyfunction!(load_path, m)?)?;
    m.add_function(wrap_pyfunction!(serialize, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_path, m)?)?;
    Ok(())
}
