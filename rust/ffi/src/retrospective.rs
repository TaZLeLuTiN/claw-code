// [SECTION:0001_types]
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Summary returned by `retrospective_generate`.
#[derive(Debug, Clone)]
pub struct RetroResult {
    pub project: String,
    pub milestone: String,
    pub period_days: u32,
    pub total_chunks_analyzed: usize,
    pub insights_proposed: usize,
    pub ollama_used: bool,
    pub pending_total: usize,
}

// [SECTION:0002_generate]

/// Generate a retrospective for a project via the harnais `RetrospectiveEngine`.
///
/// Releases the GIL during the potentially long Ollama/heuristic call (D1.3).
/// Set `use_ollama=false` to skip the LLM call and use heuristic extraction.
#[allow(clippy::needless_pass_by_value)]
#[pyfunction]
pub fn retrospective_generate(
    project: String,
    milestone: String,
    days: u32,
    use_ollama: bool,
) -> PyResult<Py<PyAny>> {
    Python::with_gil(|py| {
        crate::context_broker::get_or_init_repo(py)?;
        crate::embedder::get_or_init_embedder(py)?;

        py.allow_threads(|| {
            Python::with_gil(|py2| {
                let repo_py = crate::context_broker::CB_REPO
                    .get()
                    .ok_or_else(|| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("CB repo not init")
                    })?
                    .clone_ref(py2);
                let embedder_py = crate::embedder::EMBEDDER
                    .get()
                    .ok_or_else(|| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("embedder not init")
                    })?
                    .clone_ref(py2);

                let retro_mod = py2.import_bound("tools.accumulator.retrospective")?;
                let engine_cls = retro_mod.getattr("RetrospectiveEngine")?;
                // accumulator=None → engine creates a default KnowledgeAccumulatorRepository
                let engine = engine_cls.call1((repo_py, Option::<Py<PyAny>>::None, embedder_py))?;

                let kw = PyDict::new_bound(py2);
                kw.set_item("milestone", milestone.as_str())?;
                kw.set_item("period_days", days)?;
                kw.set_item("use_ollama", use_ollama)?;
                let result = engine.call_method("generate", (project.as_str(),), Some(&kw))?;
                Ok(result.unbind())
            })
        })
    })
}

// [SECTION:0003_rust_api]

/// Rust-internal generate returning a typed `RetroResult`.
pub fn generate_internal(
    project: String,
    milestone: String,
    days: u32,
    use_ollama: bool,
) -> PyResult<RetroResult> {
    Python::with_gil(|py| {
        let py_result = retrospective_generate(project, milestone, days, use_ollama)?;
        let d = py_result.bind(py);
        Ok(RetroResult {
            project: d.get_item("project")?.extract()?,
            milestone: d.get_item("milestone")?.extract()?,
            period_days: d.get_item("period_days")?.extract()?,
            total_chunks_analyzed: d.get_item("total_chunks_analyzed")?.extract()?,
            insights_proposed: d.get_item("insights_proposed")?.extract()?,
            ollama_used: d.get_item("ollama_used")?.extract()?,
            pending_total: d.get_item("pending_total")?.extract()?,
        })
    })
}
