use meadow as m;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
pub struct Msg {
    pub timestamp: String,
    pub topic: String,
    pub data: String,
}

impl From<m::Msg<String>> for Msg {
    fn from(msg: m::Msg<String>) -> Msg {
        Msg {
            timestamp: msg.timestamp.to_string(),
            topic: msg.topic,
            data: msg.data,
        }
    }
}

#[pymethods]
impl Msg {
    pub fn data(&self) -> String {
        self.data.clone()
    }

    pub fn topic(&self) -> String {
        self.topic.clone()
    }

    pub fn timestamp(&self) -> String {
        self.timestamp.clone()
    }
}
