use tokio::io::AsyncWriteExt;
// Tokio for async
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration}; // as TokioMutex;
                                    // Tracing for logging
use tracing::*;
// Postcard is the default de/serializer
use postcard::*;
// Multi-threading primitives
use std::sync::Arc;
// Misc other imports
use chrono::Utc;

use crate::error::{Error, HostOperation};
use crate::host::GenericStore;
use crate::prelude::*;
use std::convert::TryInto;
use std::result::Result;

/// Initiate a TCP connection with a Node
#[inline]
#[tracing::instrument]
pub async fn handshake(
    stream: TcpStream,
    max_buffer_size: usize,
    max_name_size: usize,
) -> Result<(TcpStream, String), Error> {
    // Handshake
    let mut buf = vec![0u8; max_buffer_size];
    // debug!("Starting handshake");
    let mut _name: String = String::with_capacity(max_name_size);

    stream.readable().await?;

    let n = stream.try_read_buf(&mut buf)?;
    let name = std::str::from_utf8(&buf[..n])?.to_string();

    // debug!("Returning from handshake: ({:?}, {})", &stream, &_name);
    Ok((stream, name))
}

/// Host process for handling incoming connections from Nodes
#[tracing::instrument(skip_all)]
#[inline]
pub async fn process_tcp(stream: TcpStream, mut db: sled::Db, max_buffer_size: usize) {
    let mut buf = vec![0u8; max_buffer_size];
    loop {
        if let Err(e) = stream.readable().await {
            error!("{}", e);
        }
        // dbg!(&count);
        match stream.try_read(&mut buf) {
            Ok(0) => break, // TO_DO: break or continue?
            Ok(n) => {
                if let Err(e) = stream.writable().await {
                    error!("{}", e);
                }

                let bytes = &buf[..n];
                let msg: GenericMsg = match from_bytes(bytes) {
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

                match &msg.msg_type {
                    MsgType::Subscribe => {
                        start_subscription(msg.clone(), db.clone(), &stream).await;
                    }
                    MsgType::Get => {
                        let response = match db.get_generic_nth(&msg.topic, 0) {
                            Ok(g) => g,
                            Err(e) => GenericMsg::result(Err(e)),
                        };
                        if let Ok(return_bytes) = response.as_bytes() {
                            if let Ok(()) = stream.writable().await {
                                if let Err(e) = stream.try_write(&return_bytes) {
                                    error!("Error sending data back on TCP/TOPICS: {:?}", e);
                                }
                            }
                        }

                        continue;
                    }
                    MsgType::GetNth(n) => {
                        let response = match db.get_generic_nth(&msg.topic, *n) {
                            Ok(g) => g,
                            Err(e) => GenericMsg::result(Err(e)),
                        };
                        if let Ok(return_bytes) = response.as_bytes() {
                            if let Ok(()) = stream.writable().await {
                                if let Err(e) = stream.try_write(&return_bytes) {
                                    error!("Error sending data back on TCP/TOPICS: {:?}", e);
                                }
                            }
                        }

                        continue;
                    }
                    MsgType::Set => {
                        let response = GenericMsg::result(db.insert_generic(msg));
                        if let Ok(return_bytes) = response.as_bytes() {
                            if let Ok(()) = stream.writable().await {
                                if let Err(e) = stream.try_write(&return_bytes) {
                                    error!("Error sending data back on TCP/TOPICS: {:?}", e);
                                }
                            }
                        }

                        continue;
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
                            if let Ok(()) = stream.writable().await {
                                if let Err(e) = stream.try_write(&return_bytes) {
                                    error!("Error sending data back on TCP/TOPICS: {:?}", e);
                                }
                            }
                        }

                        continue;
                    }
                    MsgType::Result(result) => match result {
                        Ok(_) => (),
                        Err(e) => error!("{}", e),
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

async fn start_subscription(msg: GenericMsg, db: sled::Db, stream: &TcpStream) {
    let specialized: Msg<Duration> = msg.clone().try_into().unwrap();
    let rate = specialized.data;

    loop {
        let response = match db.get_generic_nth(&msg.topic, 0) {
            Ok(g) => g,
            Err(e) => GenericMsg::result(Err(e)),
        };
        if let Ok(return_bytes) = response.as_bytes() {
            if let Ok(()) = stream.writable().await {
                if let Err(e) = stream.try_write(&return_bytes) {
                    error!("Error sending data back on TCP/TOPICS: {:?}", e);
                }
            }
        }
        sleep(rate).await;
    }
}
