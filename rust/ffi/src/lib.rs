// [SECTION:0001_modules]
use pyo3::prelude::*;

pub mod coherence;
pub mod context_broker;
pub mod embedder;
pub mod retrospective;
pub mod runtime;

// [SECTION:0002_pymodule]

#[pyfunction]
pub fn version() -> PyResult<String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[pymodule]
fn harnais_ffi(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    // Embedder
    m.add_function(wrap_pyfunction!(embedder::labse_embed_single, m)?)?;
    m.add_function(wrap_pyfunction!(embedder::labse_embed_batch, m)?)?;
    // Context Broker
    m.add_function(wrap_pyfunction!(context_broker::cb_ingest_chunk, m)?)?;
    m.add_function(wrap_pyfunction!(context_broker::cb_get_recent, m)?)?;
    // Retrospective
    m.add_function(wrap_pyfunction!(retrospective::retrospective_generate, m)?)?;
    // Coherence
    m.add_function(wrap_pyfunction!(coherence::mdl_score, m)?)?;
    Ok(())
}

// [SECTION:0003_rust_api]

#[must_use]
pub fn rust_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
