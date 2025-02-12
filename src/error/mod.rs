#![allow(unused_variables)]

mod host_operation;
pub use crate::error::host_operation::*;
#[cfg(feature = "quic")]
mod quic;
#[cfg(feature = "quic")]
pub use crate::error::quic::*;

use core::fmt::{Display, Formatter};
use serde::*;
use std::str::{FromStr, Utf8Error};
use thiserror::Error;

/// Meadow's Error type
#[derive(Debug, Clone, Error, Serialize, Deserialize, PartialEq)]
pub enum Error {
    /// Errors based on Host operations
    #[error(transparent)]
    HostOperation(crate::error::host_operation::HostError),
    /// No subscription value exists
    #[error("No subscription value exists")]
    NoSubscriptionValue,
    /// Couldn't achieve lock on shared resource
    #[error("Couldn't achieve lock on shared resource")]
    LockFailure,
    /// Unable to produce IP address from specified interface
    #[error("Unable to produce IP address from specified interface")]
    InvalidInterface,
    /// Transparent `sled` error
    #[error("`sled::Error`-derived error")]
    Sled(SledError),
    /// Unable to create a Tokio runtime
    #[error("Unable to create a Tokio runtime")]
    RuntimeCreation,
    /// Transparent `postcard`
    #[error(transparent)]
    Postcard(#[from] postcard::Error),
    /// Transparent std `Utf-8` error
    #[error("`std::str::Utf8Error`-derived error")]
    Utf8,
    /// Error accessing an owned `TcpStream`
    #[error("Error accessing an owned TcpStream")]
    AccessStream,
    /// Error accessing an owned `UdpSocket`
    #[error("Error accessing an owned UdpSocket")]
    AccessSocket,
    /// `TcpStream` connection attempt failure
    #[error("TcpStream connection attempt failure")]
    StreamConnection,
    /// Transparent QUIC-related errors
    #[cfg(feature = "quic")]
    #[error(transparent)]
    Quic(#[from] crate::error::quic::Quic),
    /// Transparent `std::io` error
    #[error("std::io::Error-derived error")]
    Io {
        error_kind: String,
        raw_os_error: Option<i32>,
    },
    #[error("Unable to access Tokio runtime handle")]
    HandleAccess,
    /// Topic does not exist on Host
    #[error("Topic `{0}` does not exist")]
    NonExistentTopic(String),
    /// Topic does not have value at specific n'th position
    #[error("Topic does not have value at specific n'th position")]
    NoNthValue,
    #[error("Undefined error")]
    Undefined,
}

/// This is the Result type used by meadow.
pub type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq)]
pub enum Sled {
    #[error("Collection not found")]
    CollectionNotFound,
    #[error("Unspecified sled-related error")]
    Other,
}

impl From<sled::Error> for Sled {
    fn from(value: sled::Error) -> Self {
        match value {
            sled::Error::CollectionNotFound(ivec) => Sled::CollectionNotFound,
            _ => Sled::Other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq)]
pub enum Postcard {
    #[error("Serde serialization error")]
    SerdeSerCustom,
    #[error("Serde deserialization error")]
    SerdeDeCustom,
    #[error("Other Postcard error")]
    Other,
}

impl From<postcard::Error> for Postcard {
    fn from(value: postcard::Error) -> Self {
        match value {
            postcard::Error::SerdeDeCustom => Postcard::SerdeDeCustom,
            postcard::Error::SerdeSerCustom => Postcard::SerdeSerCustom,
            _ => Postcard::Other,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        let error_kind = error.kind().to_string();

        Error::Io {
            error_kind,
            raw_os_error: error.raw_os_error(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IoError {
    kind: String,
    raw_os_error: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SledError {
    CollectionNotFound(Vec<u8>),
    Other,
}

impl From<sled::Error> for Error {
    fn from(error: sled::Error) -> Self {
        match error {
            sled::Error::CollectionNotFound(ivec) => {
                let bytes: Vec<u8> = ivec.to_vec();
                Error::Sled(SledError::CollectionNotFound(bytes))
            }
            _ => Error::Sled(SledError::Other),
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::Utf8
    }
}
