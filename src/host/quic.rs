use crate::error::{
    Error, HostOperation,
    Quic::{self, *},
};
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

pub async fn process_quic(stream: (SendStream, RecvStream), db: sled::Db, buf: &mut [u8]) {
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
                let tree = db
                    .open_tree(msg.topic.as_bytes())
                    .expect("Error opening tree");

                let db_result = match tree.insert(msg.timestamp.to_string(), bytes) {
                    Ok(_prev_msg) => crate::error::HostOperation::SUCCESS, //"SUCCESS".to_string(),
                    Err(_e) => {
                        error!("{:?}", _e);
                        crate::error::HostOperation::FAILURE
                    }
                };

                if let Ok(bytes) = postcard::to_allocvec(&db_result) {
                    for _ in 0..10 {
                        match tx.write(&bytes).await {
                            Ok(_n) => {
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
            MsgType::Get => {
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
            }
            MsgType::GetNth(n) => {
                let tree = db
                    .open_tree(msg.topic.as_bytes())
                    .expect("Error opening tree");

                match tree.iter().nth_back(n) {
                    Some(topic) => {
                        let return_bytes = match topic {
                            Ok((_timestamp, bytes)) => bytes,
                            Err(e) => {
                                let e: String =
                                    format!("Error: no topic \"{}\" exists", &msg.topic);
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
                    }
                    None => {
                        let e: String = format!("Error: no topic \"{}\" exists", &msg.topic);
                        error!("{}", &e);

                        match tx.write(&e.as_bytes()).await {
                            Ok(_n) => {}
                            Err(e) => {
                                error!("{}", e);
                            }
                        }
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
            MsgType::Topics => {
                let names = db.tree_names();
                let mut strings = Vec::new();
                for name in names {
                    if let Ok(name) = std::str::from_utf8(&name[..]) {
                        strings.push(name.to_string());
                    }
                }
                // Remove default sled tree name
                let index = strings
                    .iter()
                    .position(|x| *x == "__sled__default")
                    .unwrap();
                strings.remove(index);
                if let Ok(data) = to_allocvec(&strings) {
                    let mut packet = GenericMsg::topics();
                    packet.set_data(data);

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
