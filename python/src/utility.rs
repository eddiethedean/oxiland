use oxiland::utility::vocab::{dc, owl, rdf, rdfs, xsd};
use oxiland::utility::{
    DigestAlgorithm, Namespace, digest_bytes as rust_digest_bytes, digest_hex as rust_digest_hex,
};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyModule};

use crate::error::map_error;
use crate::terms::PyNamedNode;

#[pyclass(
    name = "DigestAlgorithm",
    module = "oxiland",
    frozen,
    skip_from_py_object
)]
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
) -> PyResult<Py<PyAny>> {
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

fn register_submodule(
    py: Python<'_>,
    parent: &Bound<'_, PyModule>,
    parent_name: &str,
    name: &str,
    pairs: &[(&str, &str)],
) -> PyResult<()> {
    let module = PyModule::new(py, name)?;
    let qualname = format!("{parent_name}.{name}");
    module.setattr("__name__", &qualname)?;
    module.setattr("__package__", &qualname)?;
    for (key, iri) in pairs {
        module.add(*key, *iri)?;
    }
    py.import("sys")?
        .getattr("modules")?
        .set_item(&qualname, &module)?;
    parent.setattr(name, &module)?;
    Ok(())
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDigestAlgorithm>()?;
    m.add_class::<PyNamespace>()?;
    m.add_function(wrap_pyfunction!(digest_hex, m)?)?;
    m.add_function(wrap_pyfunction!(digest_bytes, m)?)?;

    let py = m.py();
    let sys_modules = py.import("sys")?.getattr("modules")?;
    let vocab = PyModule::new(py, "vocab")?;
    vocab.setattr("__name__", "oxiland.vocab")?;
    vocab.setattr("__package__", "oxiland.vocab")?;
    sys_modules.set_item("oxiland.vocab", &vocab)?;

    register_submodule(
        py,
        &vocab,
        "oxiland.vocab",
        "rdf",
        &[
            ("NS", rdf::NS),
            ("type", rdf::type_().as_str()),
            ("Property", rdf::property().as_str()),
        ],
    )?;
    register_submodule(
        py,
        &vocab,
        "oxiland.vocab",
        "rdfs",
        &[
            ("NS", rdfs::NS),
            ("label", rdfs::label().as_str()),
            ("Class", rdfs::class().as_str()),
        ],
    )?;
    register_submodule(
        py,
        &vocab,
        "oxiland.vocab",
        "xsd",
        &[
            ("NS", xsd::NS),
            ("string", xsd::string().as_str()),
            ("integer", xsd::integer().as_str()),
        ],
    )?;
    register_submodule(
        py,
        &vocab,
        "oxiland.vocab",
        "owl",
        &[
            ("NS", owl::NS),
            ("Class", owl::class().as_str()),
            ("Ontology", owl::ontology().as_str()),
        ],
    )?;
    register_submodule(
        py,
        &vocab,
        "oxiland.vocab",
        "dc",
        &[
            ("NS", dc::NS),
            ("title", dc::title().as_str()),
            ("creator", dc::creator().as_str()),
        ],
    )?;

    // Maturin editable/wheel layouts may expose the extension as
    // `oxiland.oxiland` while the import root is package `oxiland`.
    m.setattr("vocab", &vocab)?;
    if let Ok(pkg) = sys_modules.get_item("oxiland") {
        let _ = pkg.setattr("vocab", &vocab);
    }
    Ok(())
}
