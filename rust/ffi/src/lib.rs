// [SECTION:0001_pymodule]
use pyo3::prelude::*;

pub mod runtime;

#[pyfunction]
pub fn version() -> PyResult<String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[pymodule]
fn harnais_ffi(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}

#[must_use]
pub fn rust_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
