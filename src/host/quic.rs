use crate::error::{
    Error, HostOperation,
    Quic::{self, *},
};
use crate::*;
use futures_util::lock::Mutex;
use futures_util::StreamExt;
use quinn::Connection as QuicConnection;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex as TokioMutex;

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
    let cert_file = match File::open::<PathBuf>(cert_path.into()) {
        Ok(file) => file,
        Err(_) => return Err(Error::Quic(FindCerts)),
    };

    let mut cert_chain_reader = BufReader::new(cert_file);
    let certs = match rustls_pemfile::certs(&mut cert_chain_reader) {
        Ok(certs) => certs,
        Err(_) => return Err(Error::Quic(ReadCerts)),
    };

    let certs = certs.into_iter().map(rustls::Certificate).collect();

    let key_file = match File::open(key_path.into()) {
        Ok(file) => file,
        Err(_) => return Err(Error::Quic(FindKeys)),
    };

    let mut key_reader = BufReader::new(key_file);
    // Since our private key file starts with "BEGIN PRIVATE KEY"
    let mut keys = match rustls_pemfile::pkcs8_private_keys(&mut key_reader) {
        Ok(keys) => keys,
        Err(_) => return Err(Error::Quic(ReadKeys)),
    };

    assert_eq!(keys.len(), 1);
    if keys.len() == 1 {
        let key = rustls::PrivateKey(keys.remove(0));

        Ok((certs, key))
    } else {
        Err(Error::Quic(ReadKeys))
    }
}

pub async fn process_quic(
    stream: (SendStream, RecvStream),
    db: sled::Db,
    buf: &mut [u8],
    count: Arc<TokioMutex<usize>>,
) {
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
        // debug!("{:?}", &msg);
        match msg.msg_type {
            MsgType::SET => {
                let tree = db
                    .open_tree(msg.topic.as_bytes())
                    .expect("Error opening tree");

                let db_result = match tree.insert(msg.timestamp.to_string(), bytes) {
                    Ok(_prev_msg) => crate::error::HostOperation::SUCCESS, //"SUCCESS".to_string(),
                    Err(_e) => crate::error::HostOperation::FAILURE,
                };

                if let Ok(bytes) = postcard::to_allocvec(&db_result) {
                    loop {
                        match tx.write(&bytes).await {
                            Ok(_n) => {
                                let mut count = count.lock().await;
                                *count += 1;
                                break;
                            }
                            Err(e) => {
                                error!("{}", e);
                                continue;
                            }
                        }
                    }
                }
            }
            MsgType::GET => {
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
                    Ok(_n) => {
                        let mut count = count.lock().await;
                        *count += 1;
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            MsgType::TOPICS => {
                let names = db.tree_names();
                let mut strings = Vec::new();
                for name in names {
                    if let Ok(name) = std::str::from_utf8(&name[..]) {
                        strings.push(name.to_string());
                    }
                }
                if let Ok(data) = to_allocvec(&strings) {
                    let packet: GenericMsg = GenericMsg {
                        msg_type: MsgType::TOPICS,
                        timestamp: Utc::now(),
                        topic: "".to_string(),
                        data_type: std::any::type_name::<Vec<String>>().to_string(),
                        data,
                    };

                    if let Ok(bytes) = to_allocvec(&packet) {
                        if let Err(e) = tx.write(&bytes).await {
                            error!("Error sending data back on QUIC/TOPICS: {:?}", e);
                        }
                    }
                }
            }
        }
    }
}
