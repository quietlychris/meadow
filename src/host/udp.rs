// Tokio for async
use tokio::net::UdpSocket;
use tokio::runtime::Handle;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration}; // as TokioMutex;
                                    // Tracing for logging
use tracing::*;
// Multi-threading primitives
use std::sync::Arc;
// Misc other imports
use chrono::Utc;

use crate::error::Error;
use crate::host::host::{GenericStore, Store};
use crate::prelude::*;
use std::convert::TryInto;
use std::net::SocketAddr;

async fn process_msg(
    msg: GenericMsg,
    socket: &UdpSocket,
    return_addr: SocketAddr,
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
                let msg = match db.get_generic(&msg.topic.to_string()) {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("{}", e);
                        GenericMsg::error(e)
                    }
                };
                if let Ok(bytes) = msg.as_bytes() {
                    if let Err(e) = socket.try_send_to(&bytes, return_addr) {
                        error!("{}", e);
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
#[tracing::instrument(skip(db))]
#[inline]
pub async fn process_udp(
    rt_handle: Handle,
    socket: UdpSocket,
    db: sled::Db,
    max_buffer_size: usize,
) {
    let mut buf = vec![0u8; max_buffer_size];
    let s = Arc::new(socket);

    // TO_DO_PART_B: Tried to with try_read_buf(), but seems to panic?
    // let mut buf = Vec::with_capacity(max_buffer_size);
    loop {
        // dbg!(&count);
        let s = s.clone();
        match s.recv_from(&mut buf).await {
            Ok((0, _)) => break, // TO_DO: break or continue?
            Ok((n, return_addr)) => {
                let bytes = &buf[..n];
                let msg: GenericMsg = match postcard::from_bytes(bytes) {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Had received Msg of {} bytes: {:?}, Error: {}", n, bytes, e);
                        continue;
                    }
                };

                let response = match process_msg(msg, &s, return_addr, db.clone()).await {
                    Ok(msg) => msg,
                    Err(e) => Some(GenericMsg::error(e)),
                };
                if let Some(msg) = response {
                    if let Ok(bytes) = msg.as_bytes() {
                        if let Err(e) = s.try_send_to(&bytes, return_addr) {
                            error!("{}", e);
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // println!("Error::WouldBlock: {:?}", e);
                continue;
            }
            Err(e) => {
                // println!("Error: {:?}", e);
                error!("Error: {:?}", e);
            }
        }
    }
}
