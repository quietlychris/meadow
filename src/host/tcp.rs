use tokio::io::AsyncWriteExt;
// Tokio for async
use tokio::net::TcpStream;
use tokio::sync::Mutex; // as TokioMutex;
use tokio::time::{sleep, Duration};
// Tracing for logging
use tracing::*;
// Postcard is the default de/serializer
use postcard::*;
// Multi-threading primitives
use std::sync::Arc;
// Misc other imports
use chrono::Utc;

use crate::error::Error;
use crate::error::*;
use crate::host::host::{GenericStore, Store};
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

#[inline]
#[tracing::instrument]
async fn process_msg(
    msg: GenericMsg,
    stream: &TcpStream,
    mut db: sled::Db,
) -> Result<Option<GenericMsg>, Error> {
    match msg.msg_type {
        MsgType::Set => {
            db.insert_generic(msg)?;
            Ok(None)
        }
        MsgType::Get => {
            // let msg: GenericMsg = db.get_generic(msg.topic)?;
            let msg = db.get_generic(msg.topic)?;
            Ok(Some(msg))
        }
        MsgType::GetNth(n) => {
            let msg = db.get_generic_nth(msg.topic, n)?;
            Ok(Some(msg))
        }
        MsgType::Subscribe => {
            let specialized: Msg<Duration> = msg.clone().try_into().unwrap();
            let rate = specialized.data;
            loop {
                if let Ok(msg) = db.get_generic(&msg.topic) {
                    if let Ok(bytes) = msg.as_bytes() {
                        if let Err(e) = stream.try_write(&bytes) {
                            error!("{}", e);
                        }
                    }
                }
                sleep(rate).await;
            }
        }
        MsgType::Topics => {
            let topics = db.topics()?;
            let msg = Msg::new(MsgType::Topics, "", topics).to_generic()?;
            Ok(Some(msg))
        }
        MsgType::Error(e) => {
            error!("Received error: {}", e);
            Ok(None)
        }
    }
}

/// Host process for handling incoming connections from Nodes
#[tracing::instrument(skip_all)]
#[inline]
pub async fn process_tcp(stream: TcpStream, db: sled::Db, max_buffer_size: usize) {
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
                match from_bytes(bytes) {
                    Ok(msg) => {
                        info!("{:?}", msg);
                        let response = match process_msg(msg, &stream, db.clone()).await {
                            Ok(msg) => {
                                if let None = msg {
                                    info!("Successfully processed received message; no response needed");
                                } else {
                                    info!(
                                        "Successfully processed received message; sending response"
                                    );
                                }
                                msg
                            }
                            Err(e) => {
                                error!("{}", e);
                                Some(GenericMsg::error(e))
                            }
                        };
                        if let Some(msg) = response {
                            if let Ok(bytes) = msg.as_bytes() {
                                if let Err(e) = stream.try_write(&bytes) {
                                    error!("{}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Had received Msg of {} bytes: {:?}, Error: {}", n, bytes, e);
                        panic!("Had received Msg of {} bytes: {:?}, Error: {}", n, bytes, e);
                    }
                };
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
