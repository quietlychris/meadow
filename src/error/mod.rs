#![allow(unused_variables)]

mod host_operation;
pub use crate::error::host_operation::*;
#[cfg(feature = "quic")]
mod quic;
#[cfg(feature = "quic")]
pub use crate::error::quic::*;

use core::fmt::{Display, Formatter};
use serde::*;

// TO_DO: These should be categorized into subgroups
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
    HostOperation(crate::error::host_operation::HostOperation),
    // General issue with QUIC setup (TO_DO: make specific error instances)
    #[cfg(feature = "quic")]
    Quic(crate::error::quic::Quic),
}

#[cfg(feature = "quic")]
impl From<crate::error::quic::Quic> for Error {
    fn from(err: crate::error::quic::Quic) -> Error {
        crate::Error::Quic(err)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Error::*;
        match self {
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
            HostOperation(_) => None,
            #[cfg(feature = "quic")]
            Quic(_) => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        use Error::*;

        match *self {
            NoSubscriptionValue => write!(f, "No subscription value exists"),
            LockFailure => write!(f, "Couldn't achieve lock on shared resource"),
            NoSled => write!(f, "The Host's sled key-value store was not found"),
            IpParsing => write!(f, "Couldn't parse the provided IP into a SocketAddr"),
            InvalidInterface => write!(f, "Unable to produce IP address from specified interface"),
            OpeningSled => write!(f, "Unable to open sled key-value store"),
            RuntimeCreation => write!(f, "Unable to create a Tokio runtime"),
            Serialization => write!(f, "Error serializing data to bytes"),
            Deserialization => write!(f, "Error deserializing data from bytes"),
            AccessStream => write!(f, "Error accessing an owned TcpStream"),
            AccessSocket => write!(f, "Error accessing an owned TcpStream"),
            BadResponse => write!(f, "Node received bad response from Host"),
            UdpSend => write!(f, "Error sending packet from UdpSocket"),
            StreamConnection => write!(f, "Error creating TcpStream"),
            Handshake => write!(f, "Error during Host <=> Node handshake"),
            HostOperation(ref err) => err.fmt(f),
            #[cfg(feature = "quic")]
            Quic(ref err) => err.fmt(f),
        }
    }
}

impl Error {
    pub fn as_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(&self).unwrap()
    }
}

/// This is the Result type used by meadow.
pub type Result<T> = ::core::result::Result<T, Error>;
