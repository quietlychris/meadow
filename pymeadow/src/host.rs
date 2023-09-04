use m::HostConfig;
use meadow as m;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
pub struct Host {
    _inner: m::Host,
}

#[pymethods]
impl Host {
    #[new]
    pub fn py_new() -> PyResult<Host> {
        if let Ok(mut host) = HostConfig::default().build() {
            host.start().unwrap();
            Ok(Host { _inner: host })
        } else {
            Err(PyValueError::new_err("Error creating Host"))
        }
    }
}
