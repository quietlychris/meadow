mod active;
mod idle;

use std::fs::File;
use std::io::BufReader;
use std::{error::Error, net::SocketAddr};

use quinn::ClientConfig;
use rustls::Certificate;

pub fn generate_client_config_from_certs() -> ClientConfig {
    let mut certs = rustls::RootCertStore::empty();
    let mut cert_chain_reader = BufReader::new(File::open("target/cert.pem").unwrap());
    let server_certs: Vec<Certificate> = rustls_pemfile::certs(&mut cert_chain_reader)
        .unwrap()
        .into_iter()
        .map(rustls::Certificate)
        .collect();
    for cert in server_certs {
        certs.add(&cert).unwrap();
    }

    ClientConfig::with_root_certificates(certs)
}
