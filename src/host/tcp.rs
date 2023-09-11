use tokio::io::AsyncWriteExt;
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
use chrono::Utc;

use crate::error::{Error, HostOperation::*};
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
                let msg: Msg<&[u8]> = match from_bytes(bytes) {
                    Ok(msg) => {
                        info!("{:?}", msg);
                        msg
                    }
                    Err(e) => {
                        error!("Had received Msg of {} bytes: {:?}, Error: {}", n, bytes, e);
                        panic!("Had received Msg of {} bytes: {:?}, Error: {}", n, bytes, e);
                    }
                };

                info!("{:?}", msg.msg_type);

                match msg.msg_type {
                    MsgType::SET => {
                        // println!("received {} bytes, to be assigned to: {}", n, &msg.name);
                        let tree = db
                            .open_tree(msg.topic.as_bytes())
                            .expect("Error opening tree");

                        let db_result = {
                            match tree.insert(msg.timestamp.to_string().as_bytes(), bytes) {
                                Ok(_prev_msg) => {
                                    info!("{:?}", msg.data);
                                    Ok(())
                                }
                                Err(_e) => Err(Error::HostOperation(SetFailure)),
                            }
                        };

                        if let Ok(bytes) = postcard::to_allocvec(&db_result) {
                            loop {
                                match stream.try_write(&bytes) {
                                    Ok(_n) => {
                                        let mut count = count.lock().await; //.unwrap();
                                        *count += 1;
                                        break;
                                    }
                                    Err(_e) => {
                                        // if e.kind() == std::io::ErrorKind::WouldBlock {}
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

                        let return_bytes = match tree.last().unwrap() {
                            Some(msg) => msg.1,
                            None => {
                                let e: String =
                                    format!("Error: no topic \"{}\" exists", &msg.topic);
                                error!("{}", &e);
                                e.as_bytes().into()
                            }
                        };
                        if let Ok(()) = stream.writable().await {
                            if let Err(e) = stream.try_write(&return_bytes) {
                                error!("Error sending data back on TCP/TOPICS: {:?}", e);
                            } else {
                                let mut count = count.lock().await; //.unwrap();
                                *count += 1;
                            }
                        }
                    }
                    MsgType::TOPICS => {
                        let names = db.tree_names();
                        let mut strings = Vec::new();
                        for name in names {
                            let name = std::str::from_utf8(&name[..]).unwrap();
                            strings.push(name.to_string());
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
                                if let Ok(()) = stream.writable().await {
                                    if let Err(e) = stream.try_write(&bytes) {
                                        error!("Error sending data back on TCP/TOPICS: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
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
