use meadow as m;
use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyclass]
pub struct Node {
    inner: m::Node<m::Tcp, meadow::Active, String>,
}

#[pymethods]
impl Node {
    #[new]
    pub fn py_new(topic: String) -> Node {
        let node = m::NodeConfig::<m::Tcp, String>::new(topic).build().unwrap();
        let node = node.activate().unwrap();
        Node { inner: node }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn pymeadow(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<Node>()?;
    Ok(())
}
