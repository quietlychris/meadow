use meadow as m;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::msg::Msg;

#[pyclass]
pub struct TcpNode {
    inner: m::Node<m::Tcp, meadow::Active, String>,
}

#[pymethods]
impl TcpNode {
    #[new]
    pub fn py_new(topic: String) -> TcpNode {
        let node = m::NodeConfig::<m::Tcp, String>::new(topic).build().unwrap();
        let node = node.activate().unwrap();
        TcpNode { inner: node }
    }

    pub fn publish(&self, value: &PyAny) -> PyResult<()> {
        if let Ok(()) = self.inner.publish(value.to_string()) {
            Ok(())
        } else {
            Err(PyValueError::new_err("Error publishing value"))
        }
    }

    pub fn request(&self) -> PyResult<Msg> {
        if let Ok(msg) = self.inner.request() {
            let msg = Msg::from(msg);
            Ok(msg)
        } else {
            Err(PyValueError::new_err("Error during request"))
        }
    }
}
