// Tokio for async
use tokio::net::TcpStream;
use tokio::sync::Mutex; // as TokioMutex;
// Tracing for logging
use hex_slice::*;
use tracing::*;
// Postcard is the default de/serializer
use postcard::*;
// Multi-threading primitives
use std::sync::Arc;
// Misc other imports

use crate::*;

/// Initiate a TCP connection with a Node
#[inline]
#[tracing::instrument]
pub async fn handshake(
    stream: TcpStream,
    max_buffer_size: usize,
    max_name_size: usize,
) -> (TcpStream, String) {
    // Handshake
    // let mut buf = [0u8; 4096];
    // TO_DO_PART_A: This seems fine, but PART_B errors out for some reason?
    let mut buf = Vec::with_capacity(max_buffer_size);
    info!("Starting handshake");
    let mut name: String = String::with_capacity(max_name_size);
    let mut count = 0;
    stream.readable().await.unwrap();
    loop {
        info!("In handshake loop");
        match stream.try_read_buf(&mut buf) {
            Ok(n) => {
                name = match std::str::from_utf8(&buf[..n]) {
                    Ok(name) => {
                        info!("Received connection from {}", &name);
                        name.to_owned()
                    }
                    Err(e) => {
                        error!("Error occurred during handshake on host-side: {} on byte string: {:?}, which in hex is: {:x}", e,&buf[..n],&buf[..n].as_hex());

                        //let emsg = format!("Error parsing the following bytes: {:?}",&buf[..n]);
                        //panic!("{}",emsg);

                        // println!("Error during handshake (Host-side): {:?}", e);
                        "HOST_CONNECTION_ERROR".to_owned()
                    }
                };
                break;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    count += 1;
                    if count > 20 {
                        error!("Host Handshake not unblocking!");
                        panic!("Stream won't unblock");
                    }
                } else {
                    error!("{:?}", e);
                    // println!("Error: {:?}", e);
                }
            }
        }
    }
    info!("Returning from handshake: ({:?}, {})", &stream, &name);
    (stream, name)
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
    let mut buf = [0u8; 10_000];
    // TO_DO_PART_B: Tried to with try_read_buf(), but seems to panic?
    // let mut buf = Vec::with_capacity(max_buffer_size);
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
                // dbg!(&msg);

                match msg.msg_type {
                    MsgType::SET => {
                        // println!("received {} bytes, to be assigned to: {}", n, &msg.name);
                        let db_result = match db.insert(msg.topic.as_bytes(), bytes) {
                            Ok(_prev_msg) => "SUCCESS".to_string(),
                            Err(e) => e.to_string(),
                        };

                        loop {
                            match stream.try_write(db_result.as_bytes()) {
                                Ok(_n) => {
                                    // println!("Successfully replied with {} bytes", n);
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
                        /*
                        println!(
                            "received {} bytes, asking for reply on topic: {}",
                            n, &msg.name
                        );*/

                        let return_bytes = match db.get(&msg.topic).unwrap() {
                            Some(msg) => msg,
                            None => {
                                let e: String =
                                    format!("Error: no message with the name {} exists", &msg.name);
                                error!("{}", &e);
                                e.as_bytes().into()
                            }
                        };

                        match stream.try_write(&return_bytes) {
                            Ok(_n) => {
                                // println!("Successfully replied with {} bytes", n);
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
                // println!("Error: {:?}", e);
                error!("Error: {:?}", e);
            }
        }
    }
}

