// [SECTION:0001_singleton]
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::sync::OnceLock;

pub(crate) static EMBEDDER: OnceLock<Py<PyAny>> = OnceLock::new();

/// Return (or lazily init) the cached `LaBSEEmbedder` Python instance.
///
/// `LaBSE` loads ~2 GB on first call; subsequent calls are O(1).
/// Set `HARNAIS_EMBEDDER_FAKE=1` to use the SHA-256 stub (no model download).
pub(crate) fn get_or_init_embedder(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    if let Some(e) = EMBEDDER.get() {
        return Ok(e.bind(py).clone());
    }
    let module = py.import_bound("tools.context_broker.embedder")?;
    let cls = module.getattr("LaBSEEmbedder")?;
    let instance = cls.call0()?;
    let _ = EMBEDDER.set(instance.clone().unbind());
    Ok(EMBEDDER
        .get()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("embedder init race"))?
        .bind(py)
        .clone())
}

// [SECTION:0002_embed_functions]

/// Embed a single text string → 768-dim L2-normalised float vector.
#[pyfunction]
pub fn labse_embed_single(text: String) -> PyResult<Vec<f32>> {
    Python::with_gil(|py| {
        let embedder = get_or_init_embedder(py)?;
        let ndarray = embedder.call_method1("embed_single", (text,))?;
        ndarray.call_method0("tolist")?.extract()
    })
}

/// Embed a batch of texts → list of 768-dim vectors.
///
/// Releases the GIL during embedding so other threads can proceed (D1.3).
#[allow(clippy::needless_pass_by_value)]
#[pyfunction]
pub fn labse_embed_batch(texts: Vec<String>) -> PyResult<Vec<Vec<f32>>> {
    Python::with_gil(|py| {
        get_or_init_embedder(py)?; // ensure singleton exists before GIL release
        py.allow_threads(|| {
            Python::with_gil(|py2| {
                let embedder = EMBEDDER
                    .get()
                    .ok_or_else(|| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                            "embedder not initialized",
                        )
                    })?
                    .bind(py2)
                    .clone();
                let py_texts = PyList::new_bound(py2, &texts);
                let result = embedder.call_method1("embed_batch", (py_texts.unbind(),))?;
                let mut out = Vec::with_capacity(texts.len());
                for item in result.iter()? {
                    let vec: Vec<f32> = item?.call_method0("tolist")?.extract()?;
                    out.push(vec);
                }
                Ok(out)
            })
        })
    })
}

// [SECTION:0003_rust_api]

/// Rust-internal `embed_single` — returns Vec<f32> without going through Python caller.
pub fn embed_single_internal(text: String) -> PyResult<Vec<f32>> {
    labse_embed_single(text)
}
