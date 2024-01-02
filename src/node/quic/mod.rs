mod active;
mod idle;
mod subscription;

use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::PathBuf;

use crate::error::Quic::*;
use crate::Error;

use quinn::ClientConfig;
use rustls::Certificate;

use tracing::*;

pub fn generate_client_config_from_certs(
    cert_path: Option<PathBuf>,
) -> Result<ClientConfig, Error> {
    let mut certs = rustls::RootCertStore::empty();

    if let Some(path) = cert_path {
        if let Ok(f) = File::open(path) {
            let mut cert_chain_reader = BufReader::new(f);
            let server_certs: Vec<Certificate> = rustls_pemfile::certs(&mut cert_chain_reader)
                .unwrap()
                .into_iter()
                .map(rustls::Certificate)
                .collect();
            for cert in server_certs {
                if let Err(e) = certs.add(&cert) {
                    error!("Error adding certificate: {:?}", e);
                }
                certs.add(&cert).unwrap();
            }

            Ok(ClientConfig::with_root_certificates(certs))
        } else {
            Err(Error::Quic(FindCerts))
        }
    } else {
        Err(Error::Quic(ReadCerts))
    }
}
