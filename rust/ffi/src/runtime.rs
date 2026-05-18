use pyo3::prelude::*;

/// Prepend harnais/tools/ to Python sys.path so embedded scripts are importable.
pub fn init_python_runtime() -> PyResult<()> {
    Python::with_gil(|py| {
        let sys = py.import_bound("sys")?;
        let path: Bound<'_, pyo3::types::PyList> = sys.getattr("path")?.extract()?;

        let harnais_home = std::env::var("HARNAIS_HOME").unwrap_or_else(|_| {
            format!(
                "{}/Documents/GitHub/harnais",
                std::env::var("HOME").unwrap_or_default()
            )
        });

        let tools_path = format!("{harnais_home}/tools");
        path.insert(0, tools_path)?;

        Ok(())
    })
}
