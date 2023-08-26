use m::HostConfig;
use meadow as m;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
pub struct Host {
    inner: m::Host,
}

#[pymethods]
impl Host {
    #[new]
    pub fn py_new() -> PyResult<Host> {
        if let Ok(mut host) = HostConfig::default().with_quic_config(None).build() {
            host.start().unwrap();
            Ok(Host { inner: host })
        } else {
            Err(PyValueError::new_err("Error creating Host"))
        }
    }
}
