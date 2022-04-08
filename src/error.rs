#![allow(unused_variables)]

use core::fmt::{Display, Formatter};

/// This is the error type used by Bissel
#[derive(Clone, Debug, Eq, PartialEq)]
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
            }
        )
    }
}

/// This is the Result type used by Bissel.
pub type Result<T> = ::core::result::Result<T, Error>;
