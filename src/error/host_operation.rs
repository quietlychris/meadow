use core::fmt::{Display, Formatter};
use serde::*;

use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq, Serialize, Deserialize)]
pub enum HostError {
    #[error("Unsuccessful Host-side SET operation")]
    SetFailure,
    #[error("Unsuccessful Host-side SET operation")]
    GetFailure,
    #[error("Unsuccessful Host connection")]
    ConnectionError,
    #[error("Topic does not exist")]
    NonExistentTopic,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum HostOperation {
    SUCCESS,
    FAILURE,
}
