use core::fmt::{Display, Formatter};
use serde::*;

use thiserror::Error;

#[cfg(feature = "quic")]
#[derive(Clone, Debug, Error, PartialEq)]
pub enum Quic {
    #[error("Error acquiring owned connection")]
    Connection,
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
