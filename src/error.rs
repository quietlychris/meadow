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

impl std::error::Error for HostOperation {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use HostOperation::*;
        match *self {
            Success => None,
            SetFailure => None,
            GetFailure => None,
            ConnectionError => None,
        }
    }
}

impl Display for HostOperation {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        use HostOperation::*;
        match *self {
            Success => write!(f, "Success Host-side operation"),
            SetFailure => write!(f, "Unsuccessful Host-side SET operation"),
            GetFailure => write!(f, "Unsuccessful Host-side SET operation"),
            ConnectionError => write!(f, "Unsuccessful Host connection"),
        }
    }
}

impl HostOperation {
    pub fn as_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(&self).unwrap()
    }
}

#[cfg(feature = "quic")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Quic {
    QuicIssue,
    BadGenericMsg,
    OpenBi,
    RecvRead,
    Connection,
    // Error accessing an owned Endpoint
    AccessEndpoint,
    EndpointConnect,
    FindKeys,
    ReadKeys,
    FindCerts,
    ReadCerts,
}

#[cfg(feature = "quic")]
impl std::error::Error for Quic {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Quic::*;
        match *self {
            QuicIssue => None,
            BadGenericMsg => None,
            OpenBi => None,
            RecvRead => None,
            Connection => None,
            // Error accessing an owned Endpoint
            AccessEndpoint => None,
            EndpointConnect => None,
            FindKeys => None,
            ReadKeys => None,
            FindCerts => None,
            ReadCerts => None,
        }
    }
}

#[cfg(feature = "quic")]
impl Display for Quic {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        use Quic::*;
        match self {
            QuicIssue => write!(f, "Generic Quic Issue"),
            BadGenericMsg => write!(f, "Error with deserialization into proper GenericMsg"),
            OpenBi => write!(f, "Error opening bidirectional stream from connection"),
            RecvRead => write!(f, "Error reading bytes from stream receiver"),
            Connection => write!(f, "Error acquiring owned connection"),
            // Error accessing an owned Endpoint
            AccessEndpoint => write!(f, "Unable to access owned Endpoint"),
            EndpointConnect => write!(f, "Unable to establish Connection to remote Endpoint"),
            FindKeys => write!(f, "Unable to find .pem key file"),
            ReadKeys => write!(f, "Error reading .pem key file"),
            FindCerts => write!(f, "Unable to find .pem certificates"),
            ReadCerts => write!(f, "Error reading .pem certificates"),
        }
    }
}

#[cfg(feature = "quic")]
impl Quic {
    pub fn as_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(&self).unwrap()
    }
}

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
    HostOperation(crate::error::HostOperation),
    // General issue with QUIC setup (TO_DO: make specific error instances)
    #[cfg(feature = "quic")]
    Quic(crate::error::Quic),
}

#[cfg(feature = "quic")]
impl From<crate::error::Quic> for Error {
    fn from(err: crate::error::Quic) -> Error {
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
