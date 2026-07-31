//! Native implementation of the Oxiland Python package.

mod error;
mod io;
mod model;
mod query;
mod terms;
mod utility;

use pyo3::prelude::*;

#[pymodule]
fn oxiland(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    error::register(m)?;
    terms::register(m)?;
    model::register(m)?;
    io::register(m)?;
    query::register(m)?;
    utility::register(m)?;
    Ok(())
}
