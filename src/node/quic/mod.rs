mod active;
mod idle;
mod subscription;

use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::PathBuf;

use crate::error::Quic::*;
use crate::prelude::*;

use quinn::ClientConfig;
use rustls::Certificate;

use tracing::*;

pub fn generate_client_config_from_certs(
    cert_path: Option<PathBuf>,
) -> Result<ClientConfig, Error> {
    let mut certs = rustls::RootCertStore::empty();

    let path = match cert_path {
        Some(path) => path,
        None => return Err(Error::Quic(NoProvidedCertPath)),
    };

    let f = File::open(path)?;
    let mut cert_chain_reader = BufReader::new(f);
    let server_certs: Vec<Certificate> = rustls_pemfile::certs(&mut cert_chain_reader)
        .unwrap()
        .into_iter()
        .map(rustls::Certificate)
        .collect();
    for cert in server_certs {
        if let Err(e) = certs.add(&cert) {
            error!("Error adding certificate: {:?}", e);
            return Err(Error::Quic(Webpki));
        }
    }

    Ok(ClientConfig::with_root_certificates(certs))
}
