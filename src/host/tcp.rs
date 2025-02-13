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
                    _ => {
                        let msg = process_msg(msg.clone(), db.clone()).unwrap();
                    }
                }

                match &msg.msg_type {
                    MsgType::Result(result) => {
                        if let Err(e) = result {
                            error!("{}", e);
                        }
                    }
                    MsgType::Set => {
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
                                Err(e) => Err(HostOperation::FAILURE),
                            }
                        };

                        if let Ok(bytes) = postcard::to_allocvec(&db_result) {
                            loop {
                                match stream.try_write(&bytes) {
                                    Ok(_n) => {
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
                    MsgType::Get => {
                        let tree = db
                            .open_tree(msg.topic.as_bytes())
                            .expect("Error opening tree");

                        if let Ok(topic) = tree.last() {
                            let return_bytes = match topic {
                                Some(msg) => {
                                    let b = msg.1.to_vec();
                                    b
                                }
                                None => {
                                    let e: String =
                                        format!("Error: no topic \"{}\" exists", &msg.topic);
                                    error!("{}", &e);
                                    // e.as_bytes().into();
                                    GenericMsg::result(Err(Error::NonExistentTopic(msg.topic)))
                                        .as_bytes()
                                        .unwrap()
                                }
                            };

                            if let Ok(()) = stream.writable().await {
                                if let Err(e) = stream.try_write(&return_bytes) {
                                    error!("Error sending data back on TCP/TOPICS: {:?}", e);
                                }
                            }
                        }
                    }
                    MsgType::GetNth(n) => {
                        let tree = db
                            .open_tree(msg.topic.as_bytes())
                            .expect("Error opening tree");

                        match tree.iter().nth_back(*n) {
                            Some(topic) => {
                                let return_bytes = match topic {
                                    Ok((_timestamp, bytes)) => bytes,
                                    Err(e) => {
                                        let e: String =
                                            format!("Error: no topic \"{}\" exists", &msg.topic);
                                        error!("{}", &e);
                                        e.as_bytes().into()
                                    }
                                };

                                if let Ok(()) = stream.writable().await {
                                    if let Err(e) = stream.try_write(&return_bytes) {
                                        error!("Error sending data back on TCP/TOPICS: {:?}", e);
                                    }
                                }
                            }
                            None => {
                                let e: String =
                                    format!("Error: no topic \"{}\" exists", &msg.topic);
                                error!("{}", &e);

                                if let Ok(()) = stream.writable().await {
                                    if let Err(e) = stream.try_write(e.as_bytes()) {
                                        error!("Error sending data back on TCP/TOPICS: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                    MsgType::Subscribe => {
                        let specialized: Msg<Duration> = msg.clone().try_into().unwrap();
                        let rate = specialized.data;

                        if let Ok(tree) = db.open_tree(msg.topic.as_bytes()) {
                            loop {
                                if let Ok(topic) = tree.last() {
                                    let return_bytes = match topic {
                                        Some(msg) => msg.1,
                                        None => {
                                            let e: String = format!(
                                                "Error: no topic \"{}\" exists",
                                                &msg.topic
                                            );
                                            error!("{}", &e);
                                            e.clone().as_bytes().into()
                                        }
                                    };

                                    if let Ok(()) = stream.writable().await {
                                        if let Err(e) = stream.try_write(&return_bytes) {
                                            error!(
                                                "Error sending data back on TCP/TOPICS: {:?}",
                                                e
                                            );
                                        }
                                    }
                                    sleep(rate).await;
                                }
                            }
                        }
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

                        match to_allocvec(&strings) {
                            Ok(data) => {
                                let mut packet = GenericMsg::topics();
                                packet.set_data(data);

                                if let Ok(bytes) = to_allocvec(&packet) {
                                    if let Ok(()) = stream.writable().await {
                                        if let Err(e) = stream.try_write(&bytes) {
                                            error!(
                                                "Error sending data back on TCP/TOPICS: {:?}",
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

    if let Ok(tree) = db.open_tree(msg.topic.as_bytes()) {
        loop {
            if let Ok(topic) = tree.last() {
                let return_bytes = match topic {
                    Some(msg) => msg.1,
                    None => {
                        let e: String = format!("Error: no topic \"{}\" exists", &msg.topic);
                        error!("{}", &e);
                        e.clone().as_bytes().into()
                    }
                };

                if let Ok(()) = stream.writable().await {
                    if let Err(e) = stream.try_write(&return_bytes) {
                        error!("Error sending data back on TCP/TOPICS: {:?}", e);
                    }
                }
                sleep(rate).await;
            }
        }
    }
}

fn process_msg(msg: GenericMsg, db: sled::Db) -> Result<Option<GenericMsg>, Error> {
    Ok(None)
}
