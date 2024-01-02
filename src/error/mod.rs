#![allow(unused_variables)]

mod host_operation;
pub use crate::error::host_operation::*;
#[cfg(feature = "quic")]
mod quic;
#[cfg(feature = "quic")]
pub use crate::error::quic::*;

use core::fmt::{Display, Formatter};
use serde::*;
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
    // The Host's sled key-value store was not found
    #[error("The Host's sled key-value store was not found")]
    NoSled,
    // Couldn't parse the provided IP into a SocketAddr
    #[error("Couldn't parse the provided IP into a SocketAddr")]
    IpParsing,
    // Unable to produce IP address from specified interface
    #[error("Unable to produce IP address from specified interface")]
    InvalidInterface,
    // Unable to open sled key-value store
    #[error("Unable to open sled key-value store")]
    OpeningSled,
    // Unable to create a Tokio runtime
    #[error("Unable to create a Tokio runtime")]
    RuntimeCreation,
    // Error serializing data to bytes
    #[error("Error serializing data to bytes")]
    Serialization,
    // Error deserializing data from bytes
    #[error("Error deserializing data from bytes")]
    Deserialization,
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
    // Error during Host <=> Node handshake
    #[error("Error during Host <=> Node handshake")]
    Handshake,
    // Result of Host-side message operation
    #[error("Result of Host-side message operation")]
    HostOperation(crate::error::host_operation::HostError),
    // General issue with QUIC setup (TO_DO: make specific error instances)
    #[cfg(feature = "quic")]
    #[error("generic quic error")]
    Quic(crate::error::quic::Quic),

    #[error("")]
    Writable(#[from] std::io::Error),
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
