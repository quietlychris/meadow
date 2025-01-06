use core::fmt::{Display, Formatter};
use serde::*;

use thiserror::Error;

/// QUIC-related errors
#[cfg(feature = "quic")]
#[derive(Clone, Debug, Error, PartialEq)]
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
    #[error(transparent)]
    ConnectError(#[from] quinn::ConnectError),
    /// Transparent `quinn::ConnectionError`
    #[error(transparent)]
    ConnectionError(#[from] quinn::ConnectionError),
    /// Transparent `quinn::WriteError`
    #[error(transparent)]
    WriteError(#[from] quinn::WriteError),
    /// Transparent `quinn::ReadError`
    #[error(transparent)]
    ReadError(#[from] quinn::ReadError),
    /// Rustls-based webpki error around adding certificate to RootCertStore
    #[error("Rustls-based webpki error around adding certificate to RootCertStore")]
    Webpki,
    /// Transparent `rustls::Error`
    #[error(transparent)]
    RustlsError(#[from] rustls::Error),
}
