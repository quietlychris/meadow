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
use crate::error::HostError;
use crate::host::host::{GenericStore, Store};
use crate::prelude::*;
use std::convert::TryInto;
use std::net::SocketAddr;

async fn process_msg(
    msg: GenericMsg,
    socket: &UdpSocket,
    return_addr: SocketAddr,
    mut db: sled::Db,
) -> GenericMsg {
    match msg.msg_type {
        MsgType::Set => match db.insert_generic(msg) {
            Ok(()) => GenericMsg::host_operation(Ok(())),
            Err(e) => {
                error!("{}", e);
                let e = match e {
                    Error::Host(e) => e,
                    _ => HostError::Set,
                };
                GenericMsg::host_operation(Err(e))
            }
        },
        MsgType::Get => match db.get_generic(msg.topic) {
            Ok(msg) => msg,
            Err(e) => {
                let e = match e {
                    Error::Host(e) => e,
                    _ => HostError::Get,
                };
                GenericMsg::host_operation(Err(e))
            }
        },
        MsgType::GetNth(n) => match db.get_generic_nth(msg.topic, n) {
            Ok(msg) => msg,
            Err(e) => {
                let e = match e {
                    Error::Host(e) => e,
                    _ => HostError::Get,
                };
                GenericMsg::host_operation(Err(e))
            }
        },
        MsgType::Subscribe => {
            let specialized: Msg<Duration> = msg.clone().try_into().unwrap();
            let rate = specialized.data;
            loop {
                let msg = match db.get_generic(&msg.topic.to_string()) {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("{}", e);
                        GenericMsg::host_operation(Err(HostError::Get))
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
        MsgType::Topics => match db.topics() {
            Ok(topics) => {
                let msg = Msg::new(MsgType::Topics, "", topics).to_generic().unwrap();

                msg
            }
            Err(e) => GenericMsg::host_operation(Err(HostError::Topics)),
        },
        MsgType::HostOperation(_host_op) => {
            error!("Shouldn't have gotten HostOperation");
            let msg = GenericMsg::host_operation(Err(HostError::RecvHostOp));
            msg
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

                let msg = process_msg(msg, &s, return_addr, db.clone()).await;
                if let Ok(bytes) = msg.as_bytes() {
                    if let Err(e) = s.try_send_to(&bytes, return_addr) {
                        error!("{}", e);
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
