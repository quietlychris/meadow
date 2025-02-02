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

use crate::error::{
    Error,
    HostOperation::{self, *},
};
use crate::host::host::GenericStore;
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

pub async fn pt(
    mut stream: TcpStream,
    db: sled::Db,
    max_buffer_size: usize,
) -> Result<(), crate::error::Error> {
    let mut buf = vec![0u8; max_buffer_size];
    let success = GenericMsg::host_operation(HostOperation::Success).as_bytes()?;
    let failure = GenericMsg::host_operation(HostOperation::Failure).as_bytes()?;
    loop {
        if let Err(e) = stream.readable().await {
            error!("{}", e);
        }
        match stream.try_read(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(n) => {
                if let Err(e) = stream.writable().await {
                    error!("{}", e);
                }

                let bytes = &buf[..n];
                let msg = from_bytes::<GenericMsg>(bytes)?;
                let op = process_msg(msg, &stream, db).await;
                match op {
                    Ok(()) => {
                        stream.try_write(&success)?;
                    }
                    Err(e) => {
                        error!("{}", e);
                        stream.try_write(&failure)?;
                    }
                }

                return Ok(());
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // println!("Error::WouldBlock: {:?}", e);
                continue;
            }
            Err(e) => {
                error!("Error: {:?}", e);
                return Err(Error::Io(e));
            }
        }
    }
}

use crate::host::Store;
async fn process_msg(
    msg: GenericMsg,
    stream: &TcpStream,
    mut db: sled::Db,
    // buf: &mut Vec<u8>,
) -> Result<(), crate::Error> {
    match msg.msg_type {
        MsgType::Set => {
            db.insert(msg.topic.clone(), msg.as_bytes()?)?;
            let msg = GenericMsg::host_operation(HostOperation::Success);
            stream.try_write(&msg.as_bytes()?)?;
        }
        MsgType::Get => {
            let msg: GenericMsg = db.get_generic(msg.topic)?;
            stream.try_write(&msg.as_bytes()?)?;
        }
        MsgType::GetNth(n) => {
            let msg = db.get_generic_nth(msg.topic, n)?;
            stream.try_write(&msg.as_bytes()?)?;
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
            let topics = db.get_topics()?;
            let msg = Msg::new(MsgType::Topics, "", topics).to_generic()?;
            stream.try_write(&msg.as_bytes()?)?;
        }
        MsgType::HostOperation(_host_op) => {
            error!("Shouldn't have gotten HostOperation");
        }
    }

    Ok(())
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
                if let Err(e) = process_msg(msg, &stream, db.clone()).await {
                    error!("{}", e);
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
