use core::fmt::{Display, Formatter};
use serde::*;

use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq, Serialize, Deserialize)]
pub enum HostOperation {
    #[error("Unsuccessful Host-side SET operation")]
    SetFailure,
    #[error("Unsuccessful Host-side SET operation")]
    GetFailure,
    #[error("Unsuccessful Host connection")]
    ConnectionError,
}
