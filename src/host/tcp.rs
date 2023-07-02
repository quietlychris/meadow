// Tokio for async
use tokio::net::TcpStream;
use tokio::sync::Mutex; // as TokioMutex;
                        // Tracing for logging
use tracing::*;
// Postcard is the default de/serializer
use postcard::*;
// Multi-threading primitives
use std::sync::Arc;
// Misc other imports

use crate::error::{Error, HostOperation};
use crate::*;
use std::result::Result;

/// Initiate a TCP connection with a Node
#[inline]
#[tracing::instrument]
pub async fn handshake(
    stream: TcpStream,
    max_buffer_size: usize,
    max_name_size: usize,
) -> Result<(TcpStream, String), crate::Error> {
    // Handshake
    let mut buf = vec![0u8; max_buffer_size];
    debug!("Starting handshake");
    let mut _name: String = String::with_capacity(max_name_size);
    let mut count = 0;
    stream.readable().await.unwrap();
    loop {
        debug!("In handshake loop");
        match stream.try_read_buf(&mut buf) {
            Ok(n) => {
                _name = match std::str::from_utf8(&buf[..n]) {
                    Ok(name) => {
                        debug!("Received connection from {}", &name);
                        name.to_owned()
                    }
                    Err(e) => {
                        error!(
                            "Error occurred during handshake on host-side: {} on byte string: {:?}",
                            e,
                            &buf[..n]
                        );
                        return Err(crate::Error::Handshake);
                    }
                };
                break;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    count += 1;
                    if count > 20 {
                        error!("Host Handshake not unblocking!");
                        return Err(crate::Error::Handshake);
                    }
                } else {
                    error!("{:?}", e);
                }
            }
        }
    }
    debug!("Returning from handshake: ({:?}, {})", &stream, &_name);
    Ok((stream, _name))
}

/// Host process for handling incoming connections from Nodes
#[tracing::instrument]
#[inline]
pub async fn process_tcp(
    stream: TcpStream,
    db: sled::Db,
    count: Arc<Mutex<usize>>,
    max_buffer_size: usize,
) {
    let mut buf = vec![0u8; max_buffer_size];
    loop {
        stream.readable().await.unwrap();
        // dbg!(&count);
        match stream.try_read(&mut buf) {
            Ok(0) => break, // TO_DO: break or continue?
            Ok(n) => {
                stream.writable().await.unwrap();

                let bytes = &buf[..n];
                let msg: GenericMsg = match from_bytes(bytes) {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Had received Msg of {} bytes: {:?}, Error: {}", n, bytes, e);
                        panic!("{}", e);
                    }
                };

                match msg.msg_type {
                    MsgType::SET => {
                        // println!("received {} bytes, to be assigned to: {}", n, &msg.name);
                        let db_result = match db.insert(msg.topic.as_bytes(), bytes) {
                            Ok(_prev_msg) => Error::HostOperation(HostOperation::Success), //"SUCCESS".to_string(),
                            Err(_e) => Error::HostOperation(HostOperation::SetFailure),
                        };

                        loop {
                            match stream.try_write(&db_result.as_bytes()) {
                                Ok(_n) => {
                                    let mut count = count.lock().await; //.unwrap();
                                    *count += 1;
                                    break;
                                }
                                Err(e) => {
                                    if e.kind() == std::io::ErrorKind::WouldBlock {}
                                    continue;
                                }
                            }
                        }
                    }
                    MsgType::GET => loop {
                        let return_bytes = match db.get(&msg.topic).unwrap() {
                            Some(msg) => msg,
                            None => {
                                let e: String =
                                    format!("Error: no topic \"{}\" exists", &msg.topic);
                                error!("{}", &e);
                                e.as_bytes().into()
                            }
                        };

                        match stream.try_write(&return_bytes) {
                            Ok(_n) => {
                                let mut count = count.lock().await; //.unwrap();
                                *count += 1;
                                break;
                            }
                            Err(e) => {
                                if e.kind() == std::io::ErrorKind::WouldBlock {}
                                continue;
                            }
                        }
                    },
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // println!("Error::WouldBlock: {:?}", e);
                continue;
            }
            Err(e) => {
                error!("Error: {:?}", e);
            }
        }
    }
}
