use core::fmt::{Display, Formatter};
use serde::*;

use thiserror::Error;

/// Errors coming from unsuccessful Host-side operations
#[derive(Clone, Debug, Error, Eq, PartialEq, Serialize, Deserialize)]
pub enum HostError {
    /// Unsuccessful SET operation
    #[error("Unsuccessful Host-side SET operation")]
    Set,
    /// Unsuccessful Host-side GET operation
    #[error("Unsuccessful Host-side GET operation")]
    Get,
    /// Unsuccessful Host connection
    #[error("Unsuccessful Host connection")]
    Connection,
    /// Unable to create list of topics
    #[error("Unable to create list of topics")]
    Topics,
    /// Topic does not exist on Host
    #[error("Topic `{0}` does not exist")]
    NonExistentTopic(String),
    /// Topic does not have value at specific n'th position
    #[error("Topic does not have value at specific n'th position")]
    NoNthValue,
    /// Hosts should not be receiving MsgType::HostOperation
    #[error("Hosts should not be receiving MsgType::HostOperation")]
    RecvHostOp,
}
