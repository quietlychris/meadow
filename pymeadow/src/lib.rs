use pyo3::prelude::*;

mod node;
pub use crate::node::*;
mod host;
pub use crate::host::*;
mod msg;
pub use crate::msg::*;

/// A Python module implemented in Rust.
#[pymodule]
fn pymeadow(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Host>()?;
    m.add_class::<TcpNode>()?;
    Ok(())
}
