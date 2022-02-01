// Tokio for async
use tokio::net::TcpListener;
use tokio::net::TcpStream;
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

use crate::msg::*;

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
    cfg: HostConfig,
    runtime: Runtime,
    connections: Arc<StdMutex<Vec<Connection>>>,
    task_listen: Option<JoinHandle<()>>,
    store: Option<sled::Db>,
    reply_count: Arc<Mutex<usize>>,
}

/// Host configuration structure
#[derive(Debug)]
pub struct HostConfig {
    interface: String,
    socket_num: usize,
    store_filename: String,
    max_buffer_size: usize,
    max_name_size: usize
}

impl HostConfig {
    /// Create a new HostConfig with all default options
    pub fn new(interface: impl Into<String>) -> HostConfig {
        HostConfig {
            interface: interface.into(),
            socket_num: 25_000,
            store_filename: "store".into(),
            max_buffer_size: 10_000,
            max_name_size: 100
        }
    }

    /// Assign a particular socket number for the Host's TcpListener
    pub fn socket_num(mut self, socket_num: usize) -> HostConfig {
        self.socket_num = socket_num;
        self
    }

    /// Change the maximum size of the buffer space allocated for received messages
    pub fn max_buffer_size(mut self, max_buffer_size: impl Into<usize>) -> HostConfig {
        self.max_buffer_size = max_buffer_size.into();
        self
    }

    /// Change the maximum size of the buffer space allocated for Node names
    pub fn max_name_size(mut self, max_name_size: impl Into<usize>) -> HostConfig {
        self.max_buffer_size = max_name_size.into();
        self
    }

    /// Change the filename of the Host's sled key-value store
    pub fn store_filename(mut self, store_filename: impl Into<String>) -> HostConfig {
        self.store_filename = store_filename.into();
        self
    }

    /// Construct a Host based on the HostConfig's parameters
    pub fn build(self) -> Result<Host, Box<dyn Error>> {
        let ip = crate::get_ip(&self.interface)?;
        println!(
            "On interface {:?}, the device IP is: {:?}",
            &self.interface, &ip
        );

        let raw_addr = ip + ":" + &self.socket_num.to_string();
        // If the address won't parse, this should panic
        let _addr: SocketAddr = raw_addr.parse().unwrap_or_else(|_| {
            panic!(
                "The provided address st
        ring, \"{}\" is invalid",
                raw_addr
            )
        });

        let runtime = tokio::runtime::Runtime::new()?;

        let connections = Arc::new(StdMutex::new(Vec::new()));

        let config = sled::Config::default()
            .path(&self.store_filename)
            .temporary(true);
        let store: sled::Db = config.open()?;

        let reply_count = Arc::new(Mutex::new(0));

        Ok(Host {
            cfg: self,
            runtime,
            connections,
            task_listen: None,
            store: Some(store),
            reply_count,
        })
    }
}

impl Host {
    /// Allow Host to begin accepting incoming connections
    #[tracing::instrument]
    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let ip = crate::get_ip(&self.cfg.interface)?;
        let raw_addr = ip + ":" + &self.cfg.socket_num.to_string();
        let addr: SocketAddr = raw_addr.parse()?;

        let (max_buffer_size, max_name_size) = (self.cfg.max_buffer_size, self.cfg.max_name_size);

        let connections_clone = self.connections.clone();

        let db = match self.store.clone() {
            Some(db) => db,
            None => {
                error!("Must open a sled database to start the Host");
                panic!("Must open a sled database to start the Host");
            }
        };

        let counter = self.reply_count.clone();

        let task_listen = self.runtime.spawn(async move {
            let listener = TcpListener::bind(addr).await.unwrap();

            loop {
                let (stream, stream_addr) = listener.accept().await.unwrap();
                // TO_DO: The handshake function is not always happy
                let (stream, name) = handshake(stream, max_buffer_size, max_name_size).await;
                info!("Host received connection from {:?}", &name);

                let db = db.clone();
                let counter = counter.clone();
                let connections = Arc::clone(&connections_clone.clone());

                let handle = tokio::spawn(async move {
                    process(stream, db, counter, max_buffer_size).await;
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

        self.task_listen = Some(task_listen);

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
async fn handshake(stream: TcpStream, max_buffer_size: usize, max_name_size: usize) -> (TcpStream, String) {
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
                        info!("Received connection from {}",&name);
                        name.to_owned()
                    },
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
                    count +=1;
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
    info!("Returning from handshake: ({:?}, {})",&stream, &name);
    (stream, name)
}

/// Host process for handling incoming connections from Nodes
#[tracing::instrument]
#[inline]
async fn process(stream: TcpStream, db: sled::Db, count: Arc<Mutex<usize>>, max_buffer_size: usize) {
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
