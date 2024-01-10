#![allow(unused_variables)]

mod host_operation;
pub use crate::error::host_operation::*;
#[cfg(feature = "quic")]
mod quic;
#[cfg(feature = "quic")]
pub use crate::error::quic::*;

use core::fmt::{Display, Formatter};
use serde::*;
use std::str::Utf8Error;
use thiserror::Error;

// TO_DO: These should be categorized into subgroups
/// This is the error type used by meadow
#[derive(Debug, Error)]
// #[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
pub enum Error {
    // No subscription value exists
    #[error("No subscription value exists")]
    NoSubscriptionValue,
    // Couldn't achieve lock on shared resource
    #[error("Couldn't achieve lock on shared resource")]
    LockFailure,
    // Couldn't parse the provided IP into a SocketAddr
    #[error("Couldn't parse the provided IP into a SocketAddr")]
    IpParsing,
    // Unable to produce IP address from specified interface
    #[error("Unable to produce IP address from specified interface")]
    InvalidInterface,
    // Error with sled database
    #[error("Error with sled database")]
    Sled(#[from] sled::Error),
    // Unable to create a Tokio runtime
    #[error("Unable to create a Tokio runtime")]
    RuntimeCreation,
    // Error with Postcard de/serialization
    #[error("Error with Postcard de/serialization")]
    Postcard(#[from] postcard::Error),
    // Error with Postcard de/serialization
    #[error("Error with Postcard de/serialization")]
    Utf8(#[from] Utf8Error),
    // Error accessing an owned TcpStream
    #[error("Error accessing an owned TcpStream")]
    AccessStream,
    // Error accessing an owned UdpSocket
    #[error("Error accessing an owned UdpSocket")]
    AccessSocket,
    // Node received bad response from Host
    #[error("Node received bad response from Host")]
    BadResponse,
    // Error sending packet from UdpSocket
    #[error("Error sending packet from UdpSocket")]
    UdpSend,
    // Error sending packet from TcpStream
    #[error("Error sending packet from TcpStream")]
    TcpSend,
    // `TcpStream` connection attempt failure
    #[error("TcpStream connection attempt failure")]
    StreamConnection,
    // Result of Host-side message operation
    #[error("Result of Host-side message operation")]
    HostOperation(crate::error::host_operation::HostError),
    // General issue with QUIC setup (TO_DO: make specific error instances)
    #[cfg(feature = "quic")]
    #[error("generic quic error")]
    Quic(crate::error::quic::Quic),
    #[error("")]
    Io(#[from] std::io::Error),
}

#[cfg(feature = "quic")]
impl From<crate::error::quic::Quic> for Error {
    fn from(err: crate::error::quic::Quic) -> Error {
        crate::Error::Quic(err)
    }
}

/* impl Error {
    pub fn as_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(&self).unwrap()
    }
} */

/// This is the Result type used by meadow.
pub type Result<T> = ::core::result::Result<T, Error>;
