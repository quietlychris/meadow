// Tokio for async
use tokio::net::TcpListener;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use tokio::sync::Mutex; // as TokioMutex;
use tokio::task::JoinHandle;
// QUIC requirements
#[cfg(feature = "quic")]
use futures_util::StreamExt;
#[cfg(feature = "quic")]
use quinn::Connection as QuicConnection;
#[cfg(feature = "quic")]
use quinn::{Endpoint, ServerConfig};
#[cfg(feature = "quic")]
use std::{fs::File, io::BufReader};

// Tracing for logging
use tracing::*;
// Multi-threading primitives
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
// Misc other imports
use std::net::SocketAddr;
use std::result::Result;

#[cfg(feature = "quic")]
use crate::host::quic::*;
use crate::host::tcp::*;
use crate::host::udp::*;
use crate::Error;
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
    pub task_listen_tcp: Option<JoinHandle<()>>,
    pub connections: Arc<StdMutex<Vec<Connection>>>,
    pub task_listen_udp: Option<JoinHandle<()>>,
    #[cfg(feature = "quic")]
    pub task_listen_quic: Option<JoinHandle<()>>,
    pub store: Option<sled::Db>,
    pub reply_count: Arc<Mutex<usize>>,
}

impl Host {
    /// Allow Host to begin accepting incoming connections
    #[tracing::instrument(skip(self))]
    pub fn start(&mut self) -> Result<(), crate::Error> {
        let connections = self.connections.clone();

        let db = match self.store.clone() {
            Some(db) => db,
            None => {
                error!("Must open a sled database to start the Host");
                return Err(Error::NoSled);
            }
        };
        let counter = self.reply_count.clone();

        // Start up the UDP process
        match &self.cfg.udp_cfg {
            None => warn!("Host has no UDP configuration"),
            Some(udp_cfg) => {
                let ip = match crate::get_ip(&udp_cfg.interface) {
                    Ok(ip) => ip,
                    Err(_e) => return Err(Error::InvalidInterface),
                };

                let raw_addr = ip + ":" + &udp_cfg.socket_num.to_string();
                let addr: SocketAddr = match raw_addr.parse() {
                    Ok(addr) => addr,
                    Err(_e) => return Err(crate::Error::IpParsing),
                };

                let db = db.clone();
                let counter = counter.clone();

                // Start the UDP listening socket
                let (max_buffer_size_udp, _max_name_size_udp) =
                    (udp_cfg.max_buffer_size, udp_cfg.max_name_size);
                let task_listen_udp = self.runtime.spawn(async move {
                    let socket = UdpSocket::bind(addr).await.unwrap();
                    process_udp(socket, db, counter, max_buffer_size_udp).await;
                });
                self.task_listen_udp = Some(task_listen_udp);
            }
        }

        // Start the TCP process
        match &self.cfg.tcp_cfg {
            None => warn!("Host has no TCP configuration"),
            Some(tcp_cfg) => {
                let ip = match crate::get_ip(&tcp_cfg.interface) {
                    Ok(ip) => ip,
                    Err(_e) => return Err(Error::InvalidInterface),
                };
                // TO_DO: This should probably be several parsing steps for IP, socket_num, and SocketAddr
                let raw_addr = ip + ":" + &tcp_cfg.socket_num.to_string();
                let addr: SocketAddr = match raw_addr.parse() {
                    Ok(addr) => addr,
                    Err(_e) => return Err(Error::IpParsing),
                };

                let (max_buffer_size_tcp, max_name_size_tcp) =
                    (tcp_cfg.max_buffer_size, tcp_cfg.max_name_size);
                let counter = counter.clone();
                let db = db.clone();
                let connections = Arc::clone(&connections);

                let task_listen_tcp = self.runtime.spawn(async move {
                    let listener = TcpListener::bind(addr).await.unwrap();
                    let connections = Arc::clone(&connections.clone());

                    loop {
                        let (stream, stream_addr) = listener.accept().await.unwrap();
                        let (stream, name) = match crate::host::tcp::handshake(
                            stream,
                            max_buffer_size_tcp,
                            max_name_size_tcp,
                        )
                        .await
                        {
                            Ok((stream, name)) => (stream, name),
                            Err(_e) => continue,
                        };
                        debug!("Host received connection from {:?}", &name);

                        let counter = counter.clone();
                        let connections = Arc::clone(&connections.clone());
                        let db = db.clone();

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

        // Start the QUIC process
        #[cfg(feature = "quic")]
        match &self.cfg.quic_cfg {
            None => warn!("Host has no QUIC configuration"),
            Some(cfg_quic) => {
                let ip = match crate::get_ip(&cfg_quic.network_cfg.interface) {
                    Ok(ip) => ip,
                    Err(_e) => return Err(Error::InvalidInterface),
                };
                // TO_DO: This should probably be several parsing steps for IP, socket_num, and SocketAddr
                let raw_addr = ip + ":" + &cfg_quic.network_cfg.socket_num.to_string();
                let addr: SocketAddr = match raw_addr.parse() {
                    Ok(addr) => addr,
                    Err(_e) => return Err(Error::IpParsing),
                };

                let (certs, key) =
                    read_certs_from_file(&cfg_quic.cert_path, &cfg_quic.key_path).unwrap();
                debug!("Successfully read in QUIC certs");

                let (max_buffer_size_quic, _max_name_size_quic) = (
                    cfg_quic.network_cfg.max_buffer_size,
                    cfg_quic.network_cfg.max_name_size,
                );
                let server_config = ServerConfig::with_single_cert(certs, key).unwrap();

                let task_listen_quic = self.runtime.spawn(async move {
                    let endpoint = Endpoint::server(server_config, addr).unwrap();
                    debug!(
                        "Waiting for incoming QUIC connection on {:?}",
                        endpoint.local_addr()
                    );
                    let connections = Arc::clone(&connections.clone());

                    loop {
                        let counter = counter.clone();
                        if let Some(conn) = endpoint.accept().await {
                            let connection = conn.await.unwrap();
                            let db = db.clone();
                            let remote_addr = connection.remote_address();

                            debug!(
                                "Received QUIC connection from {:?}",
                                &connection.remote_address()
                            );
                            let handle = tokio::spawn(async move {
                                loop {
                                    let db = db.clone();
                                    // TO_DO: Instead of having these buffers, is there a way that we can just use sled 
                                    // to hold our buffer space instead, removing the additional allocation?
                                    let mut buf = vec![0u8; max_buffer_size_quic];
                                    let counter = counter.clone();
                                    match connection.accept_bi().await {
                                        Ok((send, recv)) => {
                                            debug!("Host successfully received bi-directional stream from {}",connection.remote_address());
                                            tokio::spawn(async move {
                                                process_quic(
                                                    (send, recv),
                                                    db.clone(),
                                                    &mut buf,
                                                    counter.clone(),
                                                )
                                                .await;
                                            });
                                        }
                                        Err(_e) => {}
                                    }
                                }
                                });
                                let connection = Connection {
                                    handle,
                                    stream_addr: remote_addr,
                                    name: "TO_DO: temp".to_string(),
                                };

                                connections.lock().unwrap().push(connection);
                        }
                    }
                });

                self.task_listen_quic = Some(task_listen_quic);
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
                for conn in &*connections {
                    debug!("Aborting connection: {}", conn.name);
                    conn.handle.abort();
                }
                self.store = None;
                Ok(())
            }
            Err(_) => Err(crate::Error::LockFailure),
        }
    }

    pub fn topics(&self) -> Vec<String> {
        if let Some(db) = self.store.clone() {
            let names = db.tree_names();
            let mut strings = Vec::new();
            for name in names {
                let name = std::str::from_utf8(&name[..]).unwrap();
                strings.push(name.to_string());
            }

            strings
        } else {
            Vec::new()
        }
    }

    /// Print information about all Host connections
    #[no_mangle]
    pub fn print_connections(&mut self) -> Result<(), crate::Error> {
        println!("Connections:");
        match self.connections.lock() {
            Ok(connections) => {
                for conn in &*connections {
                    let name = conn.name.clone();
                    println!("\t- {}:{}", name, &conn.stream_addr);
                }
                Ok(())
            }
            Err(_) => Err(crate::Error::LockFailure),
        }
    }
}
