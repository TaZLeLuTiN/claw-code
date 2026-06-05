use pyo3::prelude::*;
use pyo3::types::PyList;
use std::sync::OnceLock;

static RUNTIME_INITIALIZED: OnceLock<()> = OnceLock::new();

/// Initialize Python sys.path for harnais tools and venv packages.
///
/// Idempotent — safe to call multiple times across threads.
/// sys.path receives `harnais_home` (namespace parent of `tools.*` packages)
/// and, if present, the venv site-packages for sentence-transformers etc.
pub fn init_python_runtime() -> PyResult<()> {
    if RUNTIME_INITIALIZED.get().is_some() {
        return Ok(());
    }
    Python::with_gil(|py| {
        let sys = py.import_bound("sys")?;
        let path: Bound<'_, PyList> = sys.getattr("path")?.extract()?;

        let harnais_home = std::env::var("HARNAIS_HOME").unwrap_or_else(|_| {
            format!(
                "{}/Documents/GitHub/harnais",
                std::env::var("HOME").unwrap_or_default()
            )
        });

        if !std::path::Path::new(&harnais_home).join("tools").exists() {
            return Err(PyErr::new::<pyo3::exceptions::PyImportError, _>(format!(
                "HARNAIS tools/ not found under: {harnais_home}"
            )));
        }

        // Venv site-packages (sentence-transformers, numpy, scipy…)
        let venv_site = format!("{harnais_home}/.venv/lib/python3.11/site-packages");
        if std::path::Path::new(&venv_site).exists() {
            path.insert(0, &venv_site)?;
        }

        // Harnais root → `import tools.context_broker.X` resolves via namespace pkg
        path.insert(0, &harnais_home)?;

        let _ = RUNTIME_INITIALIZED.set(());
        Ok(())
    })
}
