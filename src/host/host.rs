// Tokio for async
use tokio::net::TcpListener;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use tokio::sync::Mutex; // as TokioMutex;
use tokio::task::JoinHandle;
// Tracing for logging
use tracing::*;
// Multi-threading primitives
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
// Misc other imports
use std::net::SocketAddr;
use std::result::Result;

use crate::host::tcp::*;
use crate::host::udp::*;
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
    pub fn start(&mut self) -> Result<(), crate::Error> {
        let connections_clone = self.connections.clone();

        let db = match self.store.clone() {
            Some(db) => db,
            None => {
                error!("Must open a sled database to start the Host");
                return Err(crate::Error::NoSled);
            }
        };
        let counter = self.reply_count.clone();

        // Start up the UDP process
        match &self.cfg.udp_cfg {
            None => (),
            Some(udp_cfg) => {
                // let ip = crate::get_ip(&udp_cfg.interface)?;

                let ip = match crate::get_ip(&udp_cfg.interface) {
                    Ok(ip) => ip,
                    Err(_e) => return Err(Error::InvalidInterface),
                };

                let raw_addr = ip + ":" + &udp_cfg.socket_num.to_string();
                let addr: SocketAddr = match raw_addr.parse() {
                    Ok(addr) => addr,
                    Err(_e) => return Err(Error::IpParsing),
                };

                let db_udp = db.clone();
                let counter_udp = counter.clone();

                // Start the UDP listening socket
                let (max_buffer_size_udp, _max_name_size_udp) =
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
                let ip = match crate::get_ip(&tcp_cfg.interface) {
                    Ok(ip) => ip,
                    Err(_e) => return Err(Error::InvalidInterface),
                };
                let raw_addr = ip + ":" + &tcp_cfg.socket_num.to_string();
                let addr: SocketAddr = match raw_addr.parse() {
                    Ok(addr) => addr,
                    Err(_e) => return Err(Error::IpParsing),
                };

                let (max_buffer_size_tcp, max_name_size_tcp) =
                    (tcp_cfg.max_buffer_size, tcp_cfg.max_name_size);
                let task_listen_tcp = self.runtime.spawn(async move {
                    let listener = TcpListener::bind(addr).await.unwrap();

                    loop {
                        let (stream, stream_addr) = listener.accept().await.unwrap();
                        // TO_DO: The handshake function is not always happy
                        let (stream, name) = crate::host::tcp::handshake(
                            stream,
                            max_buffer_size_tcp,
                            max_name_size_tcp,
                        )
                        .await;
                        info!("Host received connection from {:?}", &name);

                        let db = db.clone();
                        let counter = counter.clone();
                        let connections = Arc::clone(&connections_clone.clone());

                        let handle = tokio::spawn(async move {
                            process_tcp(stream, db, counter, max_buffer_size_tcp).await;
                        });
                        let connection = Connection {
                            handle,
                            stream_addr,
                            name,
                        };

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
    pub fn stop(mut self) -> Result<(), crate::Error> {
        match self.connections.lock() {
            Ok(connections) => {
                for conn in connections.deref() {
                    info!("Aborting connection: {}", conn.name);
                    conn.handle.abort();
                }
                self.store = None;
                Ok(())
            }
            Err(_) => Err(crate::Error::LockFailure),
        }
    }

    /// Print information about all Host connections
    #[no_mangle]
    pub fn print_connections(&mut self) -> Result<(), crate::Error> {
        println!("Connections:");
        match self.connections.lock() {
            Ok(connections) => {
                for conn in connections.deref() {
                    let name = conn.name.clone();
                    println!("\t- {}:{}", name, &conn.stream_addr);
                }
                Ok(())
            }
            Err(_) => Err(crate::Error::LockFailure),
        }
    }
}
