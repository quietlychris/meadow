use crate::Error;
use futures_util::StreamExt;
use quinn::NewConnection;
use quinn::{Endpoint, ServerConfig};
use std::path::PathBuf;
use std::{fs, fs::File, io::BufReader};

pub fn read_certs_from_file(
    cert_path: impl Into<PathBuf>,
    key_path: impl Into<PathBuf>,
) -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey), crate::Error> {
    let cert_file = match File::open::<PathBuf>(cert_path.into()) {
        Ok(file) => file,
        Err(_) => return Err(Error::QuicIssue),
    };

    let mut cert_chain_reader = BufReader::new(cert_file);
    let certs = match rustls_pemfile::certs(&mut cert_chain_reader) {
        Ok(certs) => certs,
        Err(_) => return Err(Error::QuicIssue),
    };

    let certs = certs.into_iter().map(rustls::Certificate).collect();

    let key_file = match File::open(key_path.into()) {
        Ok(file) => file,
        Err(_) => return Err(Error::QuicIssue),
    };

    let mut key_reader = BufReader::new(key_file);
    // Since our private key file starts with "BEGIN PRIVATE KEY"
    let mut keys = match rustls_pemfile::pkcs8_private_keys(&mut key_reader) {
        Ok(keys) => keys,
        Err(_) => return Err(Error::QuicIssue),
    };

    assert_eq!(keys.len(), 1);
    let key = rustls::PrivateKey(keys.remove(0));

    Ok((certs, key))
}

pub fn generate_certs() -> Result<(), crate::Error> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();
    fs::write("target/cert.pem", cert_pem).expect("Error writing certificate to file");

    let priv_key_pem = cert.serialize_private_key_pem();
    fs::write("target/priv_key.pem", priv_key_pem).expect("Error writing private key to file");

    Ok(())
}
