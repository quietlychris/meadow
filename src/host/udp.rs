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
) -> Result<(), crate::Error> {
    match msg.msg_type {
        MsgType::Set => {
            db.insert_generic(msg)?;
            let msg = GenericMsg::host_operation(Ok(()));
            // socket.write(&msg.as_bytes()?)?;
            socket.try_send_to(&msg.as_bytes()?, return_addr)?;
        }
        MsgType::Get => {
            let msg: GenericMsg = db.get_generic(msg.topic)?;
            socket.try_send_to(&msg.as_bytes()?, return_addr)?;
        }
        MsgType::GetNth(n) => {
            let msg = db.get_generic_nth(msg.topic, n)?;
            socket.try_send_to(&msg.as_bytes()?, return_addr)?;
        }
        MsgType::Subscribe => {
            let specialized: Msg<Duration> = msg.clone().try_into().unwrap();
            let rate = specialized.data;
            loop {
                if let Ok(msg) = db.get_generic(&msg.topic) {
                    if let Ok(bytes) = msg.as_bytes() {
                        if let Err(e) = socket.try_send_to(&bytes, return_addr) {
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
            socket.try_send_to(&msg.as_bytes()?, return_addr)?;
        }
        MsgType::HostOperation(_host_op) => {
            error!("Shouldn't have gotten HostOperation");
        }
    }

    Ok(())
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

                if let Err(e) = process_msg(msg, &s, return_addr, db.clone()).await {
                    error!("{}", e);
                }

                /*                 match msg.msg_type {
                    MsgType::HostOperation(op) => {
                        // This should really never be received by Host
                        error!("Received HostOperation: {:?}", op);
                    }
                    MsgType::Set => {
                        info!("Received SET message: {:?}", &msg);
                        let tree = db
                            .open_tree(msg.topic.as_bytes())
                            .expect("Error opening tree");

                        let _db_result = {
                            match tree.insert(msg.timestamp.to_string().as_bytes(), bytes) {
                                Ok(_prev_msg) => {
                                    info!("{:?}", msg.data);
                                    crate::error::HostOperation::Success
                                }
                                Err(_e) => crate::error::HostOperation::Failure,
                            }
                        };
                    }
                    MsgType::Get => {
                        let tree = db
                            .open_tree(msg.topic.as_bytes())
                            .expect("Error opening tree");

                        if let Ok(topic) = tree.last() {
                            let return_bytes = match topic {
                                Some(msg) => msg.1,
                                None => {
                                    let e: String =
                                        format!("Error: no topic \"{}\" exists", &msg.topic);
                                    error!("{}", &e);
                                    e.as_bytes().into()
                                }
                            };

                            if let Ok(()) = s.writable().await {
                                if let Err(e) = s.try_send_to(&return_bytes, return_addr) {
                                    error!("Error sending data back on UDP/GET: {}", e)
                                };
                            };
                        }
                    }
                    MsgType::GetNth(n) => {
                        todo!()
                    }
                    MsgType::Subscribe => {
                        let specialized: Msg<Duration> = msg.clone().try_into().unwrap();
                        let rate = specialized.data;
                        info!("Received SUBSCRIBE message: {:?}", &msg);
                        info!("Received subscription @ rate {:?}", rate);

                        let db = db.clone();

                        rt_handle.spawn(async move {
                            loop {
                                let tree = db
                                    .open_tree(msg.topic.as_bytes())
                                    .expect("Error opening tree");
                                if let Ok(topic) = tree.last() {
                                    let return_bytes = match topic {
                                        Some(msg) => msg.1,
                                        None => {
                                            let e: String = format!(
                                                "Error: no topic \"{}\" exists",
                                                &msg.topic
                                            );
                                            error!("{}", &e);
                                            e.as_bytes().into()
                                        }
                                    };
                                    let return_msg = postcard::from_bytes::<GenericMsg>(&return_bytes);
                                    info!("Host sending return to subscriber: {:?}", return_msg);

                                    if let Ok(()) = s.writable().await {
                                        if let Err(e) = s.try_send_to(&return_bytes, return_addr) {
                                            error!("Error sending data back on UDP/GET: {}", e)
                                        };
                                    };
                                }
                                sleep(rate).await;
                            }
                        });
                    }
                    MsgType::Topics => {
                        let names = db.tree_names();

                        let mut strings = Vec::new();
                        for name in names {
                            match std::str::from_utf8(&name[..]) {
                                Ok(name) => {
                                    strings.push(name.to_string());
                                }
                                Err(_e) => {
                                    error!("Error converting topic name {:?} to UTF-8 bytes", name);
                                }
                            }
                        }
                        // Remove default sled tree name
                        let index = strings
                            .iter()
                            .position(|x| *x == "__sled__default")
                            .unwrap();
                        strings.remove(index);

                        match postcard::to_allocvec(&strings) {
                            Ok(data) => {
                                let packet: GenericMsg = GenericMsg {
                                    msg_type: MsgType::Topics,
                                    timestamp: Utc::now(),
                                    topic: "".to_string(),
                                    data_type: std::any::type_name::<Vec<String>>().to_string(),
                                    data,
                                };

                                if let Ok(bytes) = postcard::to_allocvec(&packet) {
                                    if let Ok(()) = s.writable().await {
                                        if let Err(e) = s.try_send_to(&bytes, return_addr) {
                                            error!(
                                                "Error sending data back on UDP/TOPICS: {:?}",
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("{:?}", e);
                            }
                        }
                    }
                } */
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
