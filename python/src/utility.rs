use oxiland::utility::vocab::{dc, owl, rdf, rdfs, xsd};
use oxiland::utility::{
    DigestAlgorithm, Namespace, digest_bytes as rust_digest_bytes, digest_hex as rust_digest_hex,
};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyModule};

use crate::error::map_error;
use crate::terms::PyNamedNode;

#[pyclass(name = "DigestAlgorithm", module = "oxiland", frozen)]
#[derive(Clone, Copy, Debug)]
pub struct PyDigestAlgorithm {
    inner: DigestAlgorithm,
}

#[pymethods]
impl PyDigestAlgorithm {
    #[classattr]
    const MD5: Self = Self {
        inner: DigestAlgorithm::Md5,
    };
    #[classattr]
    const SHA1: Self = Self {
        inner: DigestAlgorithm::Sha1,
    };
    #[classattr]
    const SHA256: Self = Self {
        inner: DigestAlgorithm::Sha256,
    };

    #[classmethod]
    fn from_name(_cls: &Bound<'_, pyo3::types::PyType>, name: &str) -> PyResult<Self> {
        Ok(Self {
            inner: DigestAlgorithm::from_name(name).map_err(map_error)?,
        })
    }

    #[getter]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn __repr__(&self) -> String {
        format!("DigestAlgorithm({:?})", self.inner.name())
    }
}

fn resolve_algo(algo: &Bound<'_, PyAny>) -> PyResult<DigestAlgorithm> {
    if let Ok(a) = algo.extract::<PyRef<'_, PyDigestAlgorithm>>() {
        return Ok(a.inner);
    }
    if let Ok(name) = algo.extract::<String>() {
        return DigestAlgorithm::from_name(&name).map_err(map_error);
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "algorithm must be DigestAlgorithm or str",
    ))
}

#[pyfunction]
fn digest_hex(algorithm: &Bound<'_, PyAny>, data: &Bound<'_, PyAny>) -> PyResult<String> {
    let algo = resolve_algo(algorithm)?;
    let bytes = extract_data(data)?;
    Ok(rust_digest_hex(algo, &bytes))
}

#[pyfunction]
fn digest_bytes(
    py: Python<'_>,
    algorithm: &Bound<'_, PyAny>,
    data: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let algo = resolve_algo(algorithm)?;
    let bytes = extract_data(data)?;
    Ok(PyBytes::new(py, &rust_digest_bytes(algo, &bytes))
        .into_any()
        .unbind())
}

fn extract_data(data: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    if let Ok(s) = data.extract::<String>() {
        return Ok(s.into_bytes());
    }
    if let Ok(b) = data.downcast::<PyBytes>() {
        return Ok(b.as_bytes().to_vec());
    }
    if let Ok(b) = data.extract::<Vec<u8>>() {
        return Ok(b);
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "data must be str or bytes",
    ))
}

#[pyclass(name = "Namespace", module = "oxiland", frozen)]
pub struct PyNamespace {
    inner: Namespace,
}

#[pymethods]
impl PyNamespace {
    #[new]
    fn new(prefix: &str, base: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Namespace::new(prefix, base).map_err(map_error)?,
        })
    }

    fn expand(&self, local: &str) -> PyResult<PyNamedNode> {
        Ok(PyNamedNode {
            inner: self.inner.expand(local).map_err(map_error)?,
        })
    }

    #[getter]
    fn prefix(&self) -> &str {
        self.inner.prefix()
    }

    #[getter]
    fn base(&self) -> &str {
        self.inner.base().as_str()
    }

    fn __repr__(&self) -> String {
        format!(
            "Namespace(prefix={:?}, base={:?})",
            self.inner.prefix(),
            self.inner.base().as_str()
        )
    }
}

fn add_vocab_module(
    parent: &Bound<'_, PyModule>,
    name: &str,
    pairs: &[(&str, &str)],
) -> PyResult<()> {
    let py = parent.py();
    let module = PyModule::new(py, name)?;
    for (key, iri) in pairs {
        module.add(*key, *iri)?;
    }
    parent.add_submodule(&module)?;
    // Ensure `import oxiland.vocab.rdf` style works when attached under vocab.
    Ok(())
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDigestAlgorithm>()?;
    m.add_class::<PyNamespace>()?;
    m.add_function(wrap_pyfunction!(digest_hex, m)?)?;
    m.add_function(wrap_pyfunction!(digest_bytes, m)?)?;

    let vocab = PyModule::new(m.py(), "vocab")?;
    add_vocab_module(
        &vocab,
        "rdf",
        &[
            ("NS", rdf::NS),
            ("type", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
            (
                "Property",
                "http://www.w3.org/1999/02/22-rdf-syntax-ns#Property",
            ),
        ],
    )?;
    add_vocab_module(
        &vocab,
        "rdfs",
        &[
            ("NS", rdfs::NS),
            ("label", "http://www.w3.org/2000/01/rdf-schema#label"),
            ("Class", "http://www.w3.org/2000/01/rdf-schema#Class"),
        ],
    )?;
    add_vocab_module(
        &vocab,
        "xsd",
        &[
            ("NS", xsd::NS),
            ("string", "http://www.w3.org/2001/XMLSchema#string"),
            ("integer", "http://www.w3.org/2001/XMLSchema#integer"),
        ],
    )?;
    add_vocab_module(
        &vocab,
        "owl",
        &[
            ("NS", owl::NS),
            ("Class", "http://www.w3.org/2002/07/owl#Class"),
            ("Ontology", "http://www.w3.org/2002/07/owl#Ontology"),
        ],
    )?;
    add_vocab_module(
        &vocab,
        "dc",
        &[
            ("NS", dc::NS),
            ("title", "http://purl.org/dc/terms/title"),
            ("creator", "http://purl.org/dc/terms/creator"),
        ],
    )?;
    m.add_submodule(&vocab)?;
    let _ = (
        rdf::type_(),
        rdfs::label(),
        xsd::string(),
        owl::class(),
        dc::title(),
    );
    Ok(())
}
