use crate::error::{
    Error, HostOperation,
    Quic::{self, *},
};
use crate::host::GenericStore;
use crate::prelude::*;
use futures_util::lock::Mutex;
use futures_util::StreamExt;
use quinn::Connection as QuicConnection;
use std::convert::TryInto;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

use chrono::Utc;
use postcard::from_bytes;
use postcard::to_allocvec;
use quinn::{Endpoint, RecvStream, SendStream, ServerConfig};
use std::path::PathBuf;
use std::{fs, fs::File, io::BufReader};
use tracing::*;

/// Configuration struct for generating QUIC private key and certificates
#[derive(Debug, Clone)]
pub struct QuicCertGenConfig {
    subject_alt_names: Vec<String>,
    cert_pem_path: PathBuf,
    priv_key_pem_path: PathBuf,
}

impl Default for QuicCertGenConfig {
    fn default() -> QuicCertGenConfig {
        QuicCertGenConfig {
            subject_alt_names: vec!["localhost".into()],
            cert_pem_path: "target/cert.pem".into(),
            priv_key_pem_path: "target/priv_key.pem".into(),
        }
    }
}

pub fn generate_certs(config: QuicCertGenConfig) {
    let cert = rcgen::generate_simple_self_signed(config.subject_alt_names)
        .expect("Error generating self-signed certificate");
    let cert_pem = cert.serialize_pem().expect("Error serialzing ");
    fs::write(config.cert_pem_path, cert_pem).expect("Error writing certificate to file");

    let priv_key_pem = cert.serialize_private_key_pem();
    fs::write(config.priv_key_pem_path, priv_key_pem).expect("Error writing private key to file");
}

pub fn read_certs_from_file(
    cert_path: impl Into<PathBuf>,
    key_path: impl Into<PathBuf>,
) -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey), crate::Error> {
    let cert_file = File::open::<PathBuf>(cert_path.into())?;

    let mut cert_chain_reader = BufReader::new(cert_file);
    let certs = rustls_pemfile::certs(&mut cert_chain_reader)?;

    let certs = certs.into_iter().map(rustls::Certificate).collect();

    let key_file = File::open(key_path.into())?;

    let mut key_reader = BufReader::new(key_file);
    // Since our private key file starts with "BEGIN PRIVATE KEY"
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)?;

    if keys.len() == 1 {
        let key = rustls::PrivateKey(keys.remove(0));
        Ok((certs, key))
    } else {
        Err(Error::Quic(ReadKeys))
    }
}

pub async fn process_quic(stream: (SendStream, RecvStream), mut db: sled::Db, buf: &mut [u8]) {
    let (mut tx, mut rx) = stream;

    if let Ok(Some(n)) = rx.read(buf).await {
        let bytes = &buf[..n];
        let msg: GenericMsg = match from_bytes(bytes) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Had received Msg of {} bytes: {:?}, Error: {}", n, bytes, e);
                panic!("{}", e);
            }
        };
        info!("{:?}", &msg);

        match msg.msg_type {
            MsgType::Result(result) => {
                if let Err(e) = result {
                    error!("Received {}", e);
                }
            }
            MsgType::Set => {
                let response = GenericMsg::result(db.insert_generic(msg));
                if let Ok(return_bytes) = response.as_bytes() {
                    if let Err(e) = tx.write(&return_bytes).await {
                        error!("{}", e);
                    }
                }
            }
            MsgType::Get => {
                let response = match db.get_generic_nth(&msg.topic, 0) {
                    Ok(g) => g,
                    Err(e) => GenericMsg::result(Err(e)),
                };
                if let Ok(return_bytes) = response.as_bytes() {
                    if let Err(e) = tx.write(&return_bytes).await {
                        error!("{}", e);
                    }
                }
            }
            MsgType::GetNth(n) => {
                let response = match db.get_generic_nth(&msg.topic, n) {
                    Ok(g) => g,
                    Err(e) => GenericMsg::result(Err(e)),
                };
                if let Ok(return_bytes) = response.as_bytes() {
                    if let Err(e) = tx.write(&return_bytes).await {
                        error!("{}", e);
                    }
                }
            }
            MsgType::Topics => {
                let response = match db.topics() {
                    Ok(mut topics) => {
                        topics.sort();
                        let msg = Msg::new(MsgType::Topics, "", topics);
                        match msg.to_generic() {
                            Ok(msg) => msg,
                            Err(e) => GenericMsg::result(Err(e)),
                        }
                    }
                    Err(e) => GenericMsg::result(Err(e)),
                };
                if let Ok(return_bytes) = response.as_bytes() {
                    if let Err(e) = tx.write(&return_bytes).await {
                        error!("{}", e);
                    }
                }
            }
            MsgType::Subscribe => {
                let specialized: Msg<Duration> = msg.clone().try_into().unwrap();
                let rate = specialized.data;

                loop {
                    let tree = db
                        .open_tree(msg.topic.as_bytes())
                        .expect("Error opening tree");

                    let return_bytes = match tree.last() {
                        Ok(Some(msg)) => msg.1,
                        _ => {
                            let e: String = format!("Error: no topic \"{}\" exists", &msg.topic);
                            error!("{}", &e);
                            e.as_bytes().into()
                        }
                    };

                    match tx.write(&return_bytes).await {
                        Ok(_n) => {}
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                    sleep(rate).await;
                }
            }
        }
    }
}
