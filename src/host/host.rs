use postcard::to_allocvec;
// Tokio for async
use sled::Db;
use std::time::Duration;
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
use std::convert::TryInto;
use std::net::{IpAddr, SocketAddr};

use std::result::Result;

#[cfg(feature = "quic")]
use crate::error::Quic::*;
#[cfg(feature = "quic")]
use crate::host::quic::*;

use crate::host::tcp::*;
use crate::host::udp::*;
use crate::prelude::*;
use crate::prelude::*;
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
    pub(crate) cfg: HostConfig,
    pub(crate) runtime: Runtime,
    pub(crate) task_listen_tcp: Option<JoinHandle<()>>,
    pub(crate) connections: Arc<StdMutex<Vec<Connection>>>,
    pub(crate) task_listen_udp: Option<JoinHandle<()>>,
    #[cfg(feature = "quic")]
    pub(crate) task_listen_quic: Option<JoinHandle<()>>,
    pub(crate) store: sled::Db,
}

pub trait Store {
    fn insert_msg<T: Message>(&mut self, msg: Msg<T>) -> Result<(), crate::Error>;
    fn insert<T: Message>(
        &mut self,
        topic: impl Into<String> + std::fmt::Debug,
        data: T,
    ) -> Result<(), crate::Error>;
    fn get<T: Message>(
        &self,
        topic: impl Into<String> + std::fmt::Debug,
    ) -> Result<Msg<T>, crate::Error>;
    fn get_nth_back<T: Message>(
        &self,
        topic: impl Into<String> + std::fmt::Debug,
        n: usize,
    ) -> Result<Msg<T>, crate::Error>;
    fn topics(&self) -> Result<Vec<String>, crate::Error>;
}

pub(crate) trait GenericStore {
    fn insert_generic(&mut self, msg: GenericMsg) -> Result<(), crate::Error>;
    fn get_generic(
        &self,
        topic: impl Into<String> + std::fmt::Debug,
    ) -> Result<GenericMsg, crate::Error>;
    fn get_generic_nth(
        &self,
        topic: impl Into<String> + std::fmt::Debug,
        n: usize,
    ) -> Result<GenericMsg, crate::Error>;
}

impl GenericStore for sled::Db {
    #[tracing::instrument]
    fn insert_generic(&mut self, msg: GenericMsg) -> Result<(), crate::Error> {
        let bytes = msg.as_bytes()?;
        let tree = self.open_tree(msg.topic.as_bytes())?;
        tree.insert(msg.timestamp.to_string().as_bytes(), bytes)?;
        Ok(())
    }

    #[tracing::instrument]
    fn get_generic(
        &self,
        topic: impl Into<String> + std::fmt::Debug,
    ) -> Result<GenericMsg, crate::Error> {
        let topic = topic.into();
        let tree = self.open_tree(topic.as_bytes())?;
        match tree.last()? {
            Some((_timestamp, bytes)) => {
                let msg: GenericMsg = postcard::from_bytes(&bytes)?;
                Ok(msg)
            }
            None => Err(Error::NonExistentTopic(topic.to_string())),
        }
    }

    #[tracing::instrument]
    fn get_generic_nth(
        &self,
        topic: impl Into<String> + std::fmt::Debug,
        n: usize,
    ) -> Result<GenericMsg, crate::Error> {
        let topic: String = topic.into();
        let tree = self.open_tree(topic.as_bytes())?;
        match tree.iter().nth_back(n) {
            Some(n) => match n {
                Ok((_timestamp, bytes)) => {
                    let msg: GenericMsg = postcard::from_bytes(&bytes)?;
                    Ok(msg)
                }
                Err(e) => Err(e.into()),
            },
            None => Err(Error::NoNthValue),
        }
    }
}

impl Store for sled::Db {
    /// Insert a raw `Msg<T>`
    #[inline]
    fn insert_msg<T: Message>(&mut self, msg: Msg<T>) -> Result<(), crate::Error> {
        let generic: GenericMsg = msg.try_into()?;
        self.insert_generic(generic)?;

        Ok(())
    }

    /// Insert a value using a default `Msg`
    #[inline]
    fn insert<T: Message>(
        &mut self,
        topic: impl Into<String>,
        data: T,
    ) -> Result<(), crate::Error> {
        let msg = Msg::new(MsgType::Set, topic, data);
        self.insert_msg(msg)?;
        Ok(())
    }

    /// Retrieve last message on a given topic
    #[inline]
    fn get<T: Message>(&self, topic: impl Into<String>) -> Result<Msg<T>, crate::Error> {
        let generic = self.get_generic(topic.into())?;
        let msg: Msg<T> = generic.try_into()?;
        Ok(msg)
    }

    /// Retrieve n'th message on a given topic, if it exists
    #[inline]
    fn get_nth_back<T: Message>(
        &self,
        topic: impl Into<String>,
        n: usize,
    ) -> Result<Msg<T>, crate::Error> {
        let generic = self.get_generic_nth(topic.into(), n)?;
        let msg: Msg<T> = generic.try_into()?;
        Ok(msg)
    }

    #[inline]
    fn topics(&self) -> Result<Vec<String>, crate::Error> {
        let names = self.tree_names();
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
        strings.sort();
        Ok(strings)
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        if let Some(task) = &self.task_listen_tcp {
            task.abort();
            self.task_listen_tcp = None;
        }
        if let Some(task) = &self.task_listen_udp {
            task.abort();
            self.task_listen_udp = None;
        }
        #[cfg(feature = "quic")]
        if let Some(task) = &self.task_listen_quic {
            task.abort();
            self.task_listen_quic = None;
        }
        if let Ok(mut connections) = self.connections.lock() {
            for connection in &mut *connections {
                connection.handle.abort();
            }
        }
    }
}

impl Store for Host {
    /// Insert a raw `Msg<T>`
    #[inline]
    fn insert_msg<T: Message>(&mut self, msg: Msg<T>) -> Result<(), crate::Error> {
        self.db().insert_msg(msg)
    }

    /// Insert a value using a default `Msg`
    #[inline]
    fn insert<T: Message>(
        &mut self,
        topic: impl Into<String> + std::fmt::Debug,
        data: T,
    ) -> Result<(), crate::Error> {
        self.db().insert(topic, data)
    }

    /// Retrieve last message on a given topic
    #[inline]
    fn get<T: Message>(
        &self,
        topic: impl Into<String> + std::fmt::Debug,
    ) -> Result<Msg<T>, crate::Error> {
        self.db().get(topic)
    }

    /// Retrieve n'th message on a given topic, if it exists
    #[inline]
    fn get_nth_back<T: Message>(
        &self,
        topic: impl Into<String> + std::fmt::Debug,
        n: usize,
    ) -> Result<Msg<T>, crate::Error> {
        self.db().get_nth_back(topic, n)
    }

    #[inline]
    fn topics(&self) -> Result<Vec<String>, crate::Error> {
        self.db().topics()
    }
}

impl Host {
    /// Get a reference to the `Host`'s configuration
    pub fn config(&self) -> &HostConfig {
        &self.cfg
    }

    /// Get access to the underlying `sled::Db` storage engine
    pub fn db(&self) -> Db {
        self.store.clone()
    }

    /// Access Host's Tokio `Runtime`
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    /// Allow Host to begin accepting incoming connections
    #[tracing::instrument(skip(self))]
    pub fn start(&mut self) -> Result<(), crate::Error> {
        let connections = self.connections.clone();

        let db = self.store.clone();

        // Start up the UDP process
        match &self.config().udp_cfg {
            None => warn!("Host has no UDP configuration"),
            Some(udp_cfg) => {
                let ip = match get_ip(&udp_cfg.interface) {
                    Ok(ip) => ip,
                    Err(_e) => return Err(Error::InvalidInterface),
                };

                let addr = SocketAddr::new(IpAddr::V4(ip), udp_cfg.socket_num);

                let db = db.clone();

                // Start the UDP listening socket
                let (max_buffer_size_udp, _max_name_size_udp) =
                    (udp_cfg.max_buffer_size, udp_cfg.max_name_size);
                let rt_handle = self.runtime.handle().clone();
                let task_listen_udp = self.runtime.spawn(async move {
                    match UdpSocket::bind(addr).await {
                        Ok(socket) => {
                            process_udp(rt_handle.clone(), socket, db.clone(), max_buffer_size_udp)
                                .await
                        }
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                });
                self.task_listen_udp = Some(task_listen_udp);
            }
        }

        // Start the TCP process
        match &self.config().tcp_cfg {
            None => warn!("Host has no TCP configuration"),
            Some(tcp_cfg) => {
                let ip = match get_ip(&tcp_cfg.interface) {
                    Ok(ip) => ip,
                    Err(_e) => return Err(Error::InvalidInterface),
                };

                let addr = SocketAddr::new(IpAddr::V4(ip), tcp_cfg.socket_num);

                let (max_buffer_size_tcp, max_name_size_tcp) =
                    (tcp_cfg.max_buffer_size, tcp_cfg.max_name_size);
                let db = db.clone();
                let connections = Arc::clone(&connections);

                let task_listen_tcp = self.runtime.spawn(async move {
                    if let Ok(listener) = TcpListener::bind(addr).await {
                        let connections = Arc::clone(&connections.clone());

                        loop {
                            if let Ok((stream, stream_addr)) = listener.accept().await {
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

                                let connections = Arc::clone(&connections.clone());
                                let db = db.clone();

                                let handle = tokio::spawn(async move {
                                    process_tcp(stream, db, max_buffer_size_tcp).await;
                                });
                                let connection = Connection {
                                    handle,
                                    stream_addr,
                                    name,
                                };

                                // TO_DO: We should re-evaluate how connections are stored
                                connections.lock().unwrap().push(connection);
                            }
                        }
                    }
                });

                self.task_listen_tcp = Some(task_listen_tcp);
            }
        }

        // Start the QUIC process
        #[cfg(feature = "quic")]
        match &self.config().quic_cfg {
            None => warn!("Host has no QUIC configuration"),
            Some(quic_cfg) => {
                let ip = match get_ip(&quic_cfg.network_cfg.interface) {
                    Ok(ip) => ip,
                    Err(_e) => return Err(Error::InvalidInterface),
                };

                let addr = SocketAddr::new(IpAddr::V4(ip), quic_cfg.network_cfg.socket_num);
                let (certs, key) = read_certs_from_file(&quic_cfg.cert_path, &quic_cfg.key_path)?;

                debug!("Successfully read in QUIC certs");

                let (max_buffer_size_quic, _max_name_size_quic) = (
                    quic_cfg.network_cfg.max_buffer_size,
                    quic_cfg.network_cfg.max_name_size,
                );
                let server_config = ServerConfig::with_single_cert(certs, key)
                    .map_err(|e| Into::<error::Quic>::into(e))?;

                let task_listen_quic = self.runtime.spawn(async move {
                    if let Ok(endpoint) = Endpoint::server(server_config, addr) {
                        debug!(
                            "Waiting for incoming QUIC connection on {:?}",
                            endpoint.local_addr()
                        );
                        let connections = Arc::clone(&connections.clone());
                        loop {
                            if let Some(conn) = endpoint.accept().await {
                                if let Ok(connection) = conn.await {
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
                                            match connection.accept_bi().await {
                                                Ok((send, recv)) => {
                                                    debug!("Host successfully received bi-directional stream from {}",connection.remote_address());
                                                    tokio::spawn(async move {
                                                        process_quic(
                                                            (send, recv),
                                                            db.clone(),
                                                            &mut buf,
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
    pub fn stop(&self) -> Result<(), crate::Error> {
        match self.connections.lock() {
            Ok(connections) => {
                for conn in &*connections {
                    debug!("Aborting connection: {}", conn.name);
                    conn.handle.abort();
                }
                Ok(())
            }
            Err(_) => Err(crate::Error::LockFailure),
        }
    }

    /// Print information about all Host connections
    pub fn print_connections(&mut self) -> Result<(), crate::Error> {
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

#[test]
fn host_only_generic() {
    use rand::Rng;
    let sc = SledConfig::new().temporary(true);
    let mut host = HostConfig::default().with_sled_config(sc).build().unwrap();
    let mut rng = rand::thread_rng();
    for _i in 0..10 {
        let data: usize = rng.gen_range(0..10);
        let msg = Msg::new(MsgType::Set, "test2", data);
        host.insert_msg(msg.clone()).unwrap();
        let back = host.db().get_generic("test2").unwrap();
        dbg!(&back);
        let back: Msg<usize> = back.try_into().unwrap();
        assert_eq!(msg.data, back.data);
    }
}
