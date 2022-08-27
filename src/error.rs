#![allow(unused_variables)]

use core::fmt::{Display, Formatter};
use serde::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HostOperation {
    Success,
    SetFailure,
    GetFailure,
    ConnectionError,
}

impl HostOperation {
    pub fn as_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(&self).unwrap()
    }
}

/// This is the error type used by meadow
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
pub enum Error {
    // No subscription value exists
    NoSubscriptionValue,
    // Couldn't achieve lock on shared resource
    LockFailure,
    // The Host's sled key-value store was not found
    NoSled,
    // Couldn't parse the provided IP into a SocketAddr
    IpParsing,
    // Unable to produce IP address from specified interface
    InvalidInterface,
    // Unable to open sled key-value store
    OpeningSled,
    // Unable to create a Tokio runtime
    RuntimeCreation,
    // Error serializing data to bytes
    Serialization,
    // Error deserializing data from bytes
    Deserialization,
    // Error accessing an owned TcpStream
    AccessStream,
    // Error accessing an owned UdpSocket
    AccessSocket,
    // Node received bad response from Host
    BadResponse,
    // Error sending packet from UdpSocket
    UdpSend,
    // `TcpStream` connection attempt failure
    StreamConnection,
    // Error during Host <=> Node handshake
    Handshake,
    // Result of Host-side message operation
    HostOperation(crate::error::HostOperation),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Error::*;
        match *self {
            NoSubscriptionValue => None,
            LockFailure => None,
            NoSled => None,
            IpParsing => None,
            InvalidInterface => None,
            OpeningSled => None,
            RuntimeCreation => None,
            Serialization => None,
            Deserialization => None,
            AccessStream => None,
            AccessSocket => None,
            BadResponse => None,
            UdpSend => None,
            StreamConnection => None,
            Handshake => None,
            HostOperation(crate::error::HostOperation::Success) => None,
            HostOperation(crate::error::HostOperation::SetFailure) => None,
            HostOperation(crate::error::HostOperation::GetFailure) => None,
            HostOperation(crate::error::HostOperation::ConnectionError) => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        use Error::*;
        write!(
            f,
            "{}",
            match self {
                NoSubscriptionValue => "No subscription value exists",
                LockFailure => "Couldn't achieve lock on shared resource",
                NoSled => "The Host's sled key-value store was not found",
                IpParsing => "Couldn't parse the provided IP into a SocketAddr",
                InvalidInterface => "Unable to produce IP address from specified interface",
                OpeningSled => "Unable to open sled key-value store",
                RuntimeCreation => "Unable to create a Tokio runtime",
                Serialization => "Error serializing data to bytes",
                Deserialization => "Error deserializing data from bytes",
                AccessStream => "Error accessing an owned TcpStream",
                AccessSocket => "Error accessing an owned TcpStream",
                BadResponse => "Node received bad response from Host",
                UdpSend => "Error sending packet from UdpSocket",
                StreamConnection => "Error creating TcpStream",
                Handshake => "Error during Host <=> Node handshake",
                HostOperation(crate::error::HostOperation::Success) =>
                    "Success Host-side operation",
                HostOperation(crate::error::HostOperation::SetFailure) =>
                    "Unsuccessful Host-side SET operation",
                HostOperation(crate::error::HostOperation::GetFailure) =>
                    "Unsuccessful Host-side SET operation",
                HostOperation(crate::error::HostOperation::ConnectionError) =>
                    "Unsuccessful Host connection",
            }
        )
    }
}

impl Error {
    pub fn as_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(&self).unwrap()
    }
}

/// This is the Result type used by meadow.
pub type Result<T> = ::core::result::Result<T, Error>;
