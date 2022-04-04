// Tokio for async
use tokio::net::TcpListener;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use tokio::sync::Mutex; // as TokioMutex;
use tokio::task::JoinHandle;
// Tracing for logging
use tracing::*;
// Multi-threading primitives
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
// Misc other imports
use std::error::Error;
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
                        let (stream, name) = crate::host::tcp::handshake(
                            stream,
                            max_buffer_size_tcp,
                            max_name_size_tcp,
                        )
                        .await;
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
