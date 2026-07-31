use std::path::PathBuf;

use oxiland::{Error, ParseError as OxParseError};
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyString;
use pyo3::{PyErr, PyTypeInfo, create_exception};

create_exception!(oxiland, OxilandError, PyException);
create_exception!(oxiland, InvalidRdfError, OxilandError);
create_exception!(oxiland, ParseError, OxilandError);
create_exception!(oxiland, SerializeError, OxilandError);
create_exception!(oxiland, SparqlParseError, OxilandError);
create_exception!(oxiland, SparqlEvaluationError, OxilandError);
create_exception!(oxiland, StorageError, OxilandError);
create_exception!(oxiland, UnsupportedError, OxilandError);
create_exception!(oxiland, IoError, OxilandError);
create_exception!(oxiland, OpenStoreError, OxilandError);

pub fn map_error(error: Error) -> PyErr {
    Python::with_gil(|py| map_error_py(py, error))
}

fn map_error_py(py: Python<'_>, error: Error) -> PyErr {
    match error {
        Error::InvalidRdf(message) => InvalidRdfError::new_err(message),
        Error::Parse(parse) => map_parse_error(py, parse),
        Error::Serialize(message) => SerializeError::new_err(message),
        Error::SparqlParse(message) => SparqlParseError::new_err(message),
        Error::SparqlEvaluation(message) => SparqlEvaluationError::new_err(message),
        Error::Storage(message) => StorageError::new_err(message),
        Error::Unsupported(message) => UnsupportedError::new_err(message),
        Error::Io(error) => IoError::new_err(error.to_string()),
        Error::OpenStore { path, message } => {
            let display = format!("could not open store at {}: {message}", path.display());
            match OpenStoreError::type_object(py).call1((display,)) {
                Ok(exc) => {
                    let _ = exc.setattr("path", path.display().to_string());
                    let _ = exc.setattr("message", message);
                    PyErr::from_value(exc)
                }
                Err(err) => err,
            }
        }
    }
}

fn map_parse_error(py: Python<'_>, parse: OxParseError) -> PyErr {
    let message = parse.message.clone();
    let location = parse.location.as_ref().map(|loc| {
        // SourceLocation Display is " at line…"; expose a trimmed form.
        loc.to_string().trim().to_owned()
    });
    let display = match &location {
        Some(loc) => format!("{loc}: {message}"),
        None => message.clone(),
    };
    match ParseError::type_object(py).call1((display,)) {
        Ok(exc) => {
            let _ = exc.setattr("message", message);
            match &location {
                Some(loc) => {
                    let _ = exc.setattr("location", loc.as_str());
                }
                None => {
                    let _ = exc.setattr("location", py.None());
                }
            }
            PyErr::from_value(exc)
        }
        Err(err) => err,
    }
}

pub fn path_buf(path: &Bound<'_, PyAny>) -> PyResult<PathBuf> {
    // Prefer os.fspath / PathLike conversion (PyO3 PathBuf extractor).
    if let Ok(buf) = path.extract::<PathBuf>() {
        return Ok(buf);
    }
    if let Ok(s) = path.extract::<String>() {
        return Ok(PathBuf::from(s));
    }
    if let Ok(s) = path.downcast::<PyString>() {
        return Ok(PathBuf::from(s.to_string_lossy().as_ref()));
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "path must be str or PathLike",
    ))
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("OxilandError", m.py().get_type::<OxilandError>())?;
    m.add("InvalidRdfError", m.py().get_type::<InvalidRdfError>())?;
    m.add("ParseError", m.py().get_type::<ParseError>())?;
    m.add("SerializeError", m.py().get_type::<SerializeError>())?;
    m.add("SparqlParseError", m.py().get_type::<SparqlParseError>())?;
    m.add(
        "SparqlEvaluationError",
        m.py().get_type::<SparqlEvaluationError>(),
    )?;
    m.add("StorageError", m.py().get_type::<StorageError>())?;
    m.add("UnsupportedError", m.py().get_type::<UnsupportedError>())?;
    m.add("IoError", m.py().get_type::<IoError>())?;
    m.add("OpenStoreError", m.py().get_type::<OpenStoreError>())?;
    Ok(())
}
