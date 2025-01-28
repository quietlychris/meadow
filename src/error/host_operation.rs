use core::fmt::{Display, Formatter};
use serde::*;

use thiserror::Error;

/// Errors coming from unsuccessful Host-side operations
#[derive(Clone, Debug, Error, Eq, PartialEq, Serialize, Deserialize)]
pub enum HostError {
    /// Unsuccessful SET operation
    #[error("Unsuccessful Host-side SET operation")]
    SetFailure,
    /// Unsuccessful Host-side GET operation
    #[error("Unsuccessful Host-side GET operation")]
    GetFailure,
    /// Unsuccessful Host connection
    #[error("Unsuccessful Host connection")]
    ConnectionError,
    /// Topic does not exist on Host
    #[error("Topic does not exist")]
    NonExistentTopic,
    /// Topic does not have value at specific n'th position
    #[error("Topic does not have value at specific n'th position")]
    NoNthValue,
}

/// Enum for successful/failed Host operations
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum HostOperation {
    /// Successful Host-side operation
    Success,
    /// Failed Host-side operation
    Failure,
}
