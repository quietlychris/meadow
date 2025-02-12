use core::fmt::{Display, Formatter};
use serde::*;

use crate::Error;

/// QUIC-related errors
#[cfg(feature = "quic")]
#[derive(Clone, Debug, thiserror::Error, PartialEq, Serialize, Deserialize)]
pub enum Quic {
    /// Error acquiring owned connection
    #[error("Error acquiring owned connection")]
    Connection,
    /// Error reading .pem key file
    #[error("Error reading .pem key file")]
    ReadKeys,
    /// No certificate path was provided
    #[error("No certificate path was provided")]
    NoProvidedCertPath,
    /// Transparent `quin::ConnectError`
    #[error("`quinn::ConnectError`-based error")]
    ConnectError,
    /// Transparent `quinn::ConnectionError`
    #[error("`quinn::ConnectionError`-based error")]
    ConnectionError,
    /// Transparent `quinn::WriteError`
    #[error("`quinn::WriteError`-based error")]
    WriteError,
    /// Transparent `quinn::ReadError`
    #[error("`quinn::ReadError`-based error")]
    ReadError,
    /// Rustls-based webpki error around adding certificate to RootCertStore
    #[error("Rustls-based webpki error around adding certificate to RootCertStore")]
    Webpki,
    /// Transparent `rustls::Error`
    #[error("`rustls::Error`-based error")]
    RustlsError,
}

// ===== quinn::ConnectError =====
impl From<quinn::ConnectError> for Quic {
    fn from(error: quinn::ConnectError) -> Self {
        // TO_DO: This could be more fleshed out
        Quic::ConnectError
    }
}

impl From<quinn::ConnectError> for crate::Error {
    fn from(error: quinn::ConnectError) -> Self {
        // TO_DO: This could be more fleshed out
        Error::Quic(error.into())
    }
}

// ===== quinn::ConnectionError =====
impl From<quinn::ConnectionError> for Quic {
    fn from(error: quinn::ConnectionError) -> Self {
        // TO_DO: This could be more fleshed out
        Quic::ConnectionError
    }
}

impl From<quinn::ConnectionError> for crate::Error {
    fn from(error: quinn::ConnectionError) -> Self {
        // TO_DO: This could be more fleshed out
        Error::Quic(error.into())
    }
}

// ===== quinn::WriteError =====

impl From<quinn::WriteError> for Quic {
    fn from(error: quinn::WriteError) -> Self {
        // TO_DO: This could be more fleshed out
        Quic::WriteError
    }
}

impl From<quinn::WriteError> for crate::Error {
    fn from(error: quinn::WriteError) -> Self {
        // TO_DO: This could be more fleshed out
        Error::Quic(error.into())
    }
}

// ===== quinn::ReadError =====
impl From<quinn::ReadError> for Quic {
    fn from(error: quinn::ReadError) -> Self {
        // TO_DO: This could be more fleshed out
        Quic::ReadError
    }
}

impl From<quinn::ReadError> for crate::Error {
    fn from(error: quinn::ReadError) -> Self {
        // TO_DO: This could be more fleshed out
        Error::Quic(error.into())
    }
}

// ===== rustls::Error =====

impl From<rustls::Error> for Quic {
    fn from(error: rustls::Error) -> Self {
        // TO_DO: This could be more fleshed out
        Quic::RustlsError
    }
}

impl From<rustls::Error> for crate::Error {
    fn from(error: rustls::Error) -> Self {
        // TO_DO: This could be more fleshed out
        Error::Quic(error.into())
    }
}
