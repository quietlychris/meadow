use futures_util::StreamExt;
use quinn::NewConnection;
use quinn::{Endpoint, ServerConfig};
use std::error::Error;
use std::{fs, fs::File, io::BufReader};

pub fn read_certs_from_file(
) -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey), Box<dyn Error>> {
    let mut cert_chain_reader = BufReader::new(File::open("target/cert.pem")?);
    let certs = rustls_pemfile::certs(&mut cert_chain_reader)?
        .into_iter()
        .map(rustls::Certificate)
        .collect();

    let mut key_reader = BufReader::new(File::open("target/priv_key.pem")?);
    // Since our private key file starts with "BEGIN PRIVATE KEY"
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)?;

    assert_eq!(keys.len(), 1);
    let key = rustls::PrivateKey(keys.remove(0));

    Ok((certs, key))
}

pub fn generate_certs() -> Result<(), Box<dyn Error>> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();
    fs::write("target/cert.pem", cert_pem).expect("Error writing certificate to file");

    let priv_key_pem = cert.serialize_private_key_pem();
    fs::write("target/priv_key.pem", priv_key_pem).expect("Error writing private key to file");

    Ok(())
}
