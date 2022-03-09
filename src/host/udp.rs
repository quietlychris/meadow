// Tokio for async
use tokio::net::UdpSocket;
use tokio::sync::Mutex; // as TokioMutex;
// Tracing for logging
use tracing::*;
// Postcard is the default de/serializer
use postcard::*;
// Multi-threading primitives
use std::sync::Arc;
// Misc other imports


use crate::*;

/// Host process for handling incoming connections from Nodes
#[tracing::instrument]
#[inline]
pub async fn process_udp(
    socket: UdpSocket,
    db: sled::Db,
    count: Arc<Mutex<usize>>,
    max_buffer_size: usize,
) {
    let mut buf = [0u8; 10_000];
    // TO_DO_PART_B: Tried to with try_read_buf(), but seems to panic?
    // let mut buf = Vec::with_capacity(max_buffer_size);
    loop {
        // dbg!(&count);
        match socket.recv_from(&mut buf).await {
            Ok((0, _)) => break, // TO_DO: break or continue?
            Ok((n, _)) => {
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
                        let _db_result = match db.insert(msg.topic.as_bytes(), bytes) {
                            Ok(_prev_msg) => {
                                let mut count = count.lock().await; //.unwrap();
                                *count += 1;
                                "SUCCESS".to_string()
                            }
                            Err(e) => e.to_string(),
                        };
                    }
                    MsgType::GET => loop {
                        println!("Hey, we're not doing UDP responses rn");
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
