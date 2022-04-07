#![allow(unused_variables)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No subscription value exists")]
    NoSubscriptionValue,
    #[error("Couldn't achieve lock on shared resource")]
    LockFailure,
    #[error("The Host's sled key-value store was not found")]
    NoSled,
    #[error("Couldn't parse the provided IP into a SocketAddr")]
    IpParsing,
    #[error("Unable to produce IP address from specified interface")]
    InvalidInterface,
    #[error("Unable to open sled key-value store")]
    OpeningSled,
    #[error("Unable to create a Tokio runtime")]
    RuntimeCreation,
    #[error("Error serializing data to bytes")]
    Serialization,
    #[error("Error deserializing data from bytes")]
    Deserialization,
    #[error("Error accessing an owned TcpStream")]
    AccessStream,
    #[error("Error accessing an owned UdpSocket")]
    AccessSocket,
    #[error("Node received bad response from Host")]
    BadResponse,
    #[error("Error sending packet from UdpSocket")]
    UdpSend,
}

// The following is a manual implementation of the postcard errors, which
// may be swapped out in the future, since the proc-macro used by thiserror
// may meaningfully increase compile-times for end users. However, this library
// is currently being used during initial development for ease-of-use
/*
use core::fmt::{Display, Formatter};

/// This is the error type used by Postcard
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
pub enum Error {
    /// This is a feature that PostCard will never implement
    Generic,
    /// Serde Serialization Error
    SerdeSerCustom,
    /// Serde Deserialization Error
    SerdeDeCustom,
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        use Error::*;
        write!(
            f,
            "{}",
            match self {
                Generic => "This is a basic error",
                SerdeSerCustom => "Serde Serialization Error",
                SerdeDeCustom => "Serde Deserialization Error",
            }
        )
    }
}

/// This is the Result type used by Postcard.
pub type Result<T> = ::core::result::Result<T, Error>;

impl serde::ser::Error for Error {
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        Error::SerdeSerCustom
    }
}

impl serde::de::Error for Error {
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        Error::SerdeDeCustom
    }
}

impl serde::ser::StdError for Error {}
*/
