use futures_util::lock::Mutex;
use futures_util::StreamExt;
use quinn::Connection as QuicConnection;
// use quinn::{Endpoint, ServerConfig};
use crate::error::{Error, HostOperation};
use crate::*;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex as TokioMutex;

use postcard::from_bytes;
use quinn::{Endpoint, RecvStream, SendStream, ServerConfig};
use std::path::PathBuf;
use std::{fs, fs::File, io::BufReader};
use tracing::*;

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

pub async fn process_quic(
    stream: (SendStream, RecvStream),
    db: sled::Db,
    max_buffer_size: usize,
    // buf: &mut Vec<u8>,
    count: Arc<TokioMutex<usize>>,
) {
    let (mut tx, mut rx) = stream;
    let mut buf = vec![0u8; max_buffer_size];

    if let Some(n) = rx.read(&mut buf).await.unwrap() {
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
            MsgType::SET => {
                let db_result = match db.insert(msg.topic.as_bytes(), bytes) {
                    Ok(_prev_msg) => Error::HostOperation(HostOperation::Success), //"SUCCESS".to_string(),
                    Err(_e) => Error::HostOperation(HostOperation::SetFailure),
                };
                loop {
                    match tx.write(&db_result.as_bytes()).await {
                        Ok(_n) => {
                            let mut count = count.lock().await; //.unwrap();
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
            MsgType::GET => {
                let return_bytes = match db.get(&msg.topic).unwrap() {
                    Some(msg) => msg,
                    None => {
                        let e: String = format!("Error: no topic \"{}\" exists", &msg.topic);
                        error!("{}", &e);
                        e.as_bytes().into()
                    }
                };

                match tx.write(&return_bytes).await {
                    Ok(n) => {
                        let mut count = count.lock().await; //.unwrap();
                        *count += 1;
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        }

        //let msg = std::str::from_utf8(&buf[..n]).unwrap();
        //println!("msg: {:?}", msg);
        //let reply = format!("got: {} from {}", msg, connection.remote_address());
        //if let Err(e) = tx.write_all(reply.as_bytes()).await {
        //    println!("Error: {}", e);
        // };
    }
}
