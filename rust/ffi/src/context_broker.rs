// [SECTION:0001_types]
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::sync::OnceLock;

/// Lightweight mirror of CB's `ChunkRow` for Rust consumers.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: String,
    pub category: String,
    pub content: String,
    pub heat_score: f64,
}

/// Mirror of CB's `IngestResult` for Rust consumers.
#[derive(Debug, Clone)]
pub struct IngestResult {
    pub action: String,
    pub chunk_id: Option<String>,
    pub reason: String,
    pub fingerprint: String,
}

// [SECTION:0002_singleton]

pub(crate) static CB_REPO: OnceLock<Py<PyAny>> = OnceLock::new();

/// Return (or lazily init) the `ContextBrokerRepository` Python singleton.
///
/// Uses in-memory mode when `HARNAIS_CB_DSN` is not set (safe for tests).
pub(crate) fn get_or_init_repo(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    if let Some(r) = CB_REPO.get() {
        return Ok(r.bind(py).clone());
    }
    let module = py.import_bound("tools.context_broker.repository")?;
    let cls = module.getattr("ContextBrokerRepository")?;
    let instance = if std::env::var("HARNAIS_CB_DSN").is_ok() {
        cls.call0()?
    } else {
        let kwargs = PyDict::new_bound(py);
        kwargs.set_item("in_memory", true)?;
        cls.call((), Some(&kwargs))?
    };
    let _ = CB_REPO.set(instance.clone().unbind());
    Ok(CB_REPO
        .get()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("CB repo init race"))?
        .bind(py)
        .clone())
}

// [SECTION:0003_ingest]

/// Ingest a single text chunk into the Context Broker.
///
/// Releases the GIL during the embedding + dedup decision (D1.3).
#[allow(clippy::needless_pass_by_value)]
#[pyfunction]
pub fn cb_ingest_chunk(project: String, content: String, category: String) -> PyResult<Py<PyAny>> {
    Python::with_gil(|py| {
        get_or_init_repo(py)?;
        crate::embedder::get_or_init_embedder(py)?;
        crate::coherence::get_or_init_mdl_engine(py)?;

        py.allow_threads(|| {
            Python::with_gil(|py2| {
                let repo = CB_REPO
                    .get()
                    .ok_or_else(|| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("repo not init")
                    })?
                    .clone_ref(py2);
                let embedder_py = crate::embedder::EMBEDDER
                    .get()
                    .ok_or_else(|| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("embedder not init")
                    })?
                    .clone_ref(py2);
                let engine_py = crate::coherence::MDL_ENGINE
                    .get()
                    .ok_or_else(|| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("engine not init")
                    })?
                    .clone_ref(py2);

                let chunker = py2.import_bound("tools.context_broker.chunker")?;
                let chunk_cls = chunker.getattr("Chunk")?;
                let chunk_mode = chunker.getattr("ChunkMode")?.getattr("PARAGRAPH")?.unbind();
                let kwargs = PyDict::new_bound(py2);
                kwargs.set_item("content", content.as_str())?;
                kwargs.set_item("category", category.as_str())?;
                kwargs.set_item("mode", chunk_mode)?;
                let chunk_py = chunk_cls.call((), Some(&kwargs))?.unbind();

                let result = repo.call_method1(
                    py2,
                    "ingest_chunk",
                    (&project, chunk_py, embedder_py, engine_py),
                )?;
                Ok(result)
            })
        })
    })
}

// [SECTION:0004_get_recent]

/// Retrieve recent chunks from the Context Broker.
#[allow(clippy::needless_pass_by_value)]
#[pyfunction]
pub fn cb_get_recent(
    project: String,
    days: i64,
    categories: Option<Vec<String>>,
) -> PyResult<Py<PyAny>> {
    Python::with_gil(|py| {
        let repo = get_or_init_repo(py)?;
        let result = if let Some(cats) = categories {
            let py_cats = PyList::new_bound(py, &cats);
            repo.call_method1("get_recent", (project.as_str(), days, py_cats.unbind()))?
        } else {
            repo.call_method1("get_recent", (project.as_str(), days))?
        };
        Ok(result.unbind())
    })
}

// [SECTION:0005_rust_api]

/// Rust-internal `ingest_chunk` returning a typed `IngestResult`.
pub fn ingest_internal(
    project: String,
    content: String,
    category: String,
) -> PyResult<IngestResult> {
    Python::with_gil(|py| {
        let py_result = cb_ingest_chunk(project, content, category)?;
        let d = py_result.bind(py);
        let chunk_id_obj = d.getattr("chunk_id")?;
        let chunk_id = if chunk_id_obj.is_none() {
            None
        } else {
            Some(chunk_id_obj.str()?.to_str()?.to_owned())
        };
        Ok(IngestResult {
            action: d.getattr("action")?.extract()?,
            chunk_id,
            reason: d.getattr("reason")?.extract()?,
            fingerprint: d.getattr("fingerprint")?.extract()?,
        })
    })
}

fn extract_chunk_row(row: &Bound<'_, PyAny>) -> PyResult<Chunk> {
    Ok(Chunk {
        id: row.getattr("id")?.str()?.to_str()?.to_owned(),
        category: row.getattr("category")?.extract()?,
        content: row.getattr("content")?.extract()?,
        heat_score: row.getattr("heat_score")?.extract()?,
    })
}

/// Rust-internal `get_recent` returning typed `Chunk` structs.
pub fn get_recent_internal(
    project: String,
    days: i64,
    categories: Option<Vec<String>>,
) -> PyResult<Vec<Chunk>> {
    Python::with_gil(|py| {
        let py_result = cb_get_recent(project, days, categories)?;
        let result = py_result.bind(py);
        let mut chunks = Vec::new();
        for item in result.iter()? {
            chunks.push(extract_chunk_row(&item?)?);
        }
        Ok(chunks)
    })
}
