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

#[derive(Debug, Error)]
pub enum Error {
    #[error("No subscription value exists")]
    NoSubscriptionValue,
    #[error("Couldn't achieve lock on shared resource")]
    LockFailure,
    #[error("Unable to produce IP address from specified interface")]
    InvalidInterface,
    #[error(transparent)]
    Sled(#[from] sled::Error),
    #[error("Unable to create a Tokio runtime")]
    RuntimeCreation,
    #[error(transparent)]
    Postcard(#[from] postcard::Error),
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
    #[error("Error accessing an owned TcpStream")]
    AccessStream,
    #[error("Error accessing an owned UdpSocket")]
    AccessSocket,
    #[error("TcpStream connection attempt failure")]
    StreamConnection,
    #[error(transparent)]
    HostOperation(crate::error::host_operation::HostError),
    #[cfg(feature = "quic")]
    #[error(transparent)]
    Quic(#[from] crate::error::quic::Quic),
    #[error("")]
    Io(#[from] std::io::Error),
}

/* #[cfg(feature = "quic")]
impl From<crate::Error::Quic::Quic> for Error {
    fn from(err: crate::Error::Quic::Quic) -> Error {
        crate::Error::Quic(err)
    }
} */

/* impl Error {
    pub fn as_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(&self).unwrap()
    }
} */

/// This is the Result type used by meadow.
pub type Result<T> = ::core::result::Result<T, Error>;
