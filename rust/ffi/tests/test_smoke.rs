use pyo3::prelude::*;
use pyo3::types::PyModule;

#[test]
fn test_version_round_trip() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let m = PyModule::new_bound(py, "harnais_ffi").unwrap();
        m.add_function(wrap_pyfunction!(harnais_ffi::version, &m).unwrap())
            .unwrap();
        let v: String = m
            .getattr("version")
            .unwrap()
            .call0()
            .unwrap()
            .extract()
            .unwrap();
        assert_eq!(v, "0.1.0");
    });
}

#[test]
fn test_rust_version_direct() {
    assert_eq!(harnais_ffi::rust_version(), "0.1.0");
}

#[test]
fn test_py_allow_threads_releases_gil() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let result = py.allow_threads(|| {
            std::thread::sleep(std::time::Duration::from_millis(50));
            42_i32
        });
        assert_eq!(result, 42);
    });
}
