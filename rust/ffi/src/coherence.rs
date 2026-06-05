// [SECTION:0001_singleton]
use pyo3::prelude::*;
use std::sync::OnceLock;

pub(crate) static MDL_ENGINE: OnceLock<Py<PyAny>> = OnceLock::new();

/// Return (or lazily init) the cached `MDLDeduplicationEngine` Python instance.
pub(crate) fn get_or_init_mdl_engine(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    if let Some(e) = MDL_ENGINE.get() {
        return Ok(e.bind(py).clone());
    }
    let module = py.import_bound("tools.context_broker.coherence")?;
    let cls = module.getattr("MDLDeduplicationEngine")?;
    let instance = cls.call0()?; // default thresholds
    let _ = MDL_ENGINE.set(instance.clone().unbind());
    Ok(MDL_ENGINE
        .get()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("MDL engine init race"))?
        .bind(py)
        .clone())
}

// [SECTION:0002_score_function]

/// Compute MDL novelty score of vec1 relative to vec2 → float in \[0.0, 1.0\].
///
/// `score = 0.6 × cosine_novelty + 0.4 × entropy_gain − encoding_cost`
/// Returns 1.0 when vec2 is empty (unconditional novelty).
#[allow(clippy::needless_pass_by_value)]
#[pyfunction]
pub fn mdl_score(vec1: Vec<f32>, vec2: Vec<f32>) -> PyResult<f32> {
    Python::with_gil(|py| {
        let engine = get_or_init_mdl_engine(py)?;
        let np = py.import_bound("numpy")?;
        let arr1 = np.call_method1("array", (vec1,))?;
        let arr2 = np.call_method1("array", (vec2,))?;
        let neighbors = vec![arr2.unbind()];
        engine
            .call_method1("mdl_score", (arr1.unbind(), neighbors))?
            .extract()
    })
}
