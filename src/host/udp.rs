// Tokio for async
use tokio::net::UdpSocket;
use tokio::runtime::Handle;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration}; // as TokioMutex;
                                    // Tracing for logging
use tracing::*;
// Postcard is the default de/serializer
use crate::error::Error;
use postcard::*;
// Multi-threading primitives
use std::sync::Arc;
// Misc other imports
use chrono::Utc;

use crate::host::{GenericStore, Storage};
use crate::prelude::*;
use std::convert::TryInto;

/// Host process for handling incoming connections from Nodes
#[tracing::instrument(skip(db))]
#[inline]
pub async fn process_udp(
    rt_handle: Handle,
    socket: UdpSocket,
    mut db: Storage,
    max_buffer_size: usize,
) {
    let mut buf = vec![0u8; max_buffer_size];
    let s = Arc::new(socket);

    loop {
        // dbg!(&count);
        let s = s.clone();
        match s.recv_from(&mut buf).await {
            Ok((0, _)) => break, // TO_DO: break or continue?
            Ok((n, return_addr)) => {
                let bytes = &buf[..n];
                let msg: GenericMsg = match from_bytes(bytes) {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Had received Msg of {} bytes: {:?}, Error: {}", n, bytes, e);
                        continue;
                    }
                };

                match msg.msg_type {
                    MsgType::Set => {
                        if let Err(e) = db.insert_generic(msg) {
                            error!("{}", e);
                        }
                    }
                    MsgType::Get => {
                        let response = match db.get_generic_nth(&msg.topic, 0) {
                            Ok(g) => g,
                            Err(e) => GenericMsg::result(Err(e)),
                        };

                        if let Ok(return_bytes) = response.as_bytes() {
                            if let Ok(()) = s.writable().await {
                                if let Err(e) = s.try_send_to(&return_bytes, return_addr) {
                                    error!("Error sending data back on UDP/GET: {}", e)
                                };
                            };
                        }
                    }
                    MsgType::GetNth(n) => {
                        let response = match db.get_generic_nth(&msg.topic, n) {
                            Ok(g) => g,
                            Err(e) => GenericMsg::result(Err(e)),
                        };

                        if let Ok(return_bytes) = response.as_bytes() {
                            if let Ok(()) = s.writable().await {
                                if let Err(e) = s.try_send_to(&return_bytes, return_addr) {
                                    error!("Error sending data back on UDP/GET: {}", e)
                                };
                            };
                        }
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
                            if let Ok(()) = s.writable().await {
                                if let Err(e) = s.try_send_to(&return_bytes, return_addr) {
                                    error!("Error sending data back on UDP/GET: {}", e)
                                };
                            };
                        }
                    }
                    MsgType::Subscribe => {
                        let specialized: Msg<Duration> = msg.clone().try_into().unwrap();
                        let rate = specialized.data;

                        let db = db.clone();
                        rt_handle.spawn(async move {
                            loop {
                                let response = match db.get_generic_nth(&msg.topic, 0) {
                                    Ok(g) => g,
                                    Err(e) => GenericMsg::result(Err(e)),
                                };

                                if let Ok(return_bytes) = response.as_bytes() {
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
                    _ => {}
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
