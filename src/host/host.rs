// Tokio for async
use tokio::net::UdpSocket;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::Mutex; // as TokioMutex;
use tokio::task::JoinHandle;
// Tracing for logging
use hex_slice::*;
use tracing::*;
// Postcard is the default de/serializer
use postcard::*;
// Multi-threading primitives
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
// Misc other imports
use std::error::Error;
use std::net::SocketAddr;
use std::result::Result;

use crate::*;

/// Named task handle for each Hosted connection
#[derive(Debug)]
pub struct Connection {
    handle: JoinHandle<()>,
    stream_addr: SocketAddr,
    name: String,
}

/// Central coordination process, which stores published data and responds to requests
#[derive(Debug)]
pub struct Host {
    pub cfg: HostConfig,
    pub runtime: Runtime,
    pub connections: Arc<StdMutex<Vec<Connection>>>,
    pub task_listen_tcp: Option<JoinHandle<()>>,
    pub task_listen_udp: Option<JoinHandle<()>>,
    pub store: Option<sled::Db>,
    pub reply_count: Arc<Mutex<usize>>,
}

impl Host {
    /// Allow Host to begin accepting incoming connections
    #[tracing::instrument]
    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let connections_clone = self.connections.clone();

        let db = match self.store.clone() {
            Some(db) => db,
            None => {
                error!("Must open a sled database to start the Host");
                panic!("Must open a sled database to start the Host");
            }
        };
        let counter = self.reply_count.clone();

        // Start up the UDP process
        match &self.cfg.udp_cfg {
            None => (),
            Some(udp_cfg) => {
                let ip = crate::get_ip(&udp_cfg.interface)?;
                let raw_addr = ip + ":" + &udp_cfg.socket_num.to_string();
                let addr: SocketAddr = raw_addr.parse()?;

                let db_udp = db.clone();
                let counter_udp = counter.clone();

                // Start the UDP listening socket
                let (max_buffer_size_udp, max_name_size_udp) =
                    (udp_cfg.max_buffer_size, udp_cfg.max_name_size);
                let task_listen_udp = self.runtime.spawn(async move {
                    let socket = UdpSocket::bind(addr).await.unwrap();
                    process_udp(socket, db_udp, counter_udp, max_buffer_size_udp).await;
                });
                self.task_listen_udp = Some(task_listen_udp);
            }
        }

        // Start the TCP process
        match &self.cfg.tcp_cfg {
            None => (),
            Some(tcp_cfg) => {
                let ip = crate::get_ip(&tcp_cfg.interface)?;
                let raw_addr = ip + ":" + &tcp_cfg.socket_num.to_string();
                let addr: SocketAddr = raw_addr.parse()?;

                let db_tcp = db.clone();
                let counter_tcp = counter.clone();

                let (max_buffer_size_tcp, max_name_size_tcp) =
                    (tcp_cfg.max_buffer_size, tcp_cfg.max_name_size);
                let task_listen_tcp = self.runtime.spawn(async move {
                    let listener = TcpListener::bind(addr).await.unwrap();

                    loop {
                        let (stream, stream_addr) = listener.accept().await.unwrap();
                        // TO_DO: The handshake function is not always happy
                        let (stream, name) =
                            handshake(stream, max_buffer_size_tcp, max_name_size_tcp).await;
                        info!("Host received connection from {:?}", &name);

                        let db = db_tcp.clone();
                        let counter = counter_tcp.clone();
                        let connections = Arc::clone(&connections_clone.clone());

                        let handle = tokio::spawn(async move {
                            process_tcp(stream, db, counter, max_buffer_size_tcp).await;
                        });
                        let connection = Connection {
                            handle,
                            stream_addr,
                            name,
                        };
                        // dbg!(&connection);

                        connections.lock().unwrap().push(connection);
                    }
                });

                self.task_listen_tcp = Some(task_listen_tcp);
            }
        }

        Ok(())
    }

    /// Shuts down all networking connections and releases Host object handle
    /// This also makes sure that temporary sled::Db's built are also dropped
    /// following the shutdown of a Host
    #[tracing::instrument]
    pub fn stop(mut self) -> Result<(), Box<dyn Error>> {
        for conn in self.connections.lock().unwrap().deref_mut() {
            // println!("Aborting connection: {}", conn.name);
            info!("Aborting connection: {}", conn.name);
            conn.handle.abort();
        }
        self.store = None;

        Ok(())
    }

    /// Print information about all Host connections
    #[no_mangle]
    pub fn print_connections(&mut self) -> Result<(), Box<dyn Error + '_>> {
        println!("Connections:");
        for conn in self.connections.lock()?.deref() {
            let name = conn.name.clone();
            println!("\t- {}:{}", name, &conn.stream_addr);
        }
        Ok(())
    }
}

/// Initiate a connection with a Node
#[inline]
#[tracing::instrument]
async fn handshake(
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
async fn process_tcp(
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

/// Host process for handling incoming connections from Nodes
#[tracing::instrument]
#[inline]
async fn process_udp(
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
