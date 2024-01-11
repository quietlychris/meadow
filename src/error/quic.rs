use core::fmt::{Display, Formatter};
use serde::*;

use thiserror::Error;

#[cfg(feature = "quic")]
#[derive(Clone, Debug, Error, PartialEq)]
pub enum Quic {
    #[error("Generic Quic Issue")]
    QuicIssue,
    #[error("Error with deserialization into proper GenericMsg")]
    BadGenericMsg,
    #[error("Error opening bidirectional stream from connection")]
    OpenBi,
    // #[error("Error reading bytes from stream receiver")]
    // RecvRead,
    #[error("Error acquiring owned connection")]
    Connection,
    // Error accessing an owned Endpoint
    #[error("Unable to access owned Endpoint")]
    AccessEndpoint,
    #[error("Unable to establish Connection to remote Endpoint")]
    EndpointConnect,
    #[error("Error reading .pem key file")]
    ReadKeys,
    #[error("No certificate path was provided")]
    NoProvidedCertPath,
    #[error(transparent)]
    ConnectError(#[from] quinn::ConnectError),
    #[error(transparent)]
    ConnectionError(#[from] quinn::ConnectionError),
    #[error(transparent)]
    WriteError(#[from] quinn::WriteError),
    #[error(transparent)]
    ReadError(#[from] quinn::ReadError),
    #[error("Rustls-based webpki error around adding certificate to RootCertStore")]
    Webpki,
    #[error(transparent)]
    RustlsError(#[from] rustls::Error),
}
