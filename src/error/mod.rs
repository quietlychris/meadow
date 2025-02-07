#![allow(unused_variables)]

mod host;
pub use crate::error::host::*;
#[cfg(feature = "quic")]
mod quic;
#[cfg(feature = "quic")]
pub use crate::error::quic::*;

use core::fmt::{Display, Formatter};
use serde::*;
use std::str::{FromStr, Utf8Error};
use thiserror::Error;

/// Meadow's Error type
#[derive(Debug, Error)]
pub enum Error {
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
    #[error(transparent)]
    Sled(#[from] sled::Error),
    /// Unable to create a Tokio runtime
    #[error("Unable to create a Tokio runtime")]
    RuntimeCreation,
    /// Transparent `postcard`
    #[error(transparent)]
    Postcard(#[from] postcard::Error),
    /// Transparent std `Utf-8` error
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
    /// Error accessing an owned `TcpStream`
    #[error("Error accessing an owned TcpStream")]
    AccessStream,
    /// Error accessing an owned `UdpSocket`
    #[error("Error accessing an owned UdpSocket")]
    AccessSocket,
    /// `TcpStream` connection attempt failure
    #[error("TcpStream connection attempt failure")]
    StreamConnection,
    /// Errors based on Host operations
    #[error(transparent)]
    Host(#[from] crate::error::HostError),
    /// Transparent QUIC-related errors
    #[cfg(feature = "quic")]
    #[error(transparent)]
    Quic(#[from] crate::error::quic::Quic),
    /// Transparent `std::io` error
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Unable to access Tokio runtime handle")]
    HandleAccess,
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
