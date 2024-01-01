use core::fmt::{Display, Formatter};
use serde::*;

use thiserror::Error;

#[cfg(feature = "quic")]
#[derive(Clone, Debug, Error, Eq, PartialEq, Serialize, Deserialize)]
pub enum Quic {
    #[error("Generic Quic Issue")]
    QuicIssue,
    #[error("Error with deserialization into proper GenericMsg")]
    BadGenericMsg,
    #[error("Error opening bidirectional stream from connection")]
    OpenBi,
    #[error("Error reading bytes from stream receiver")]
    RecvRead,
    #[error("Error acquiring owned connection")]
    Connection,
    // Error accessing an owned Endpoint
    #[error("Unable to access owned Endpoint")]
    AccessEndpoint,
    #[error("Unable to establish Connection to remote Endpoint")]
    EndpointConnect,
    #[error("Unable to find .pem key file")]
    FindKeys,
    #[error("Error reading .pem key file")]
    ReadKeys,
    #[error("Unable to find .pem certificates")]
    FindCerts,
    #[error("Error reading .pem certificates")]
    ReadCerts,
    #[error("Error configuring server with certificate")]
    Configuration,
    #[error("Error creating endpoint from configuration parameters")]
    EndpointCreation,
}

#[cfg(feature = "quic")]
impl Quic {
    pub fn as_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(&self).unwrap()
    }
}
