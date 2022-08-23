mod active;
mod config;
mod idle;
mod network_config;
mod subscription;

pub use crate::node::active::*;
pub use crate::node::config::*;
pub use crate::node::idle::*;
pub use crate::node::network_config::*;
pub use crate::node::subscription::*;

extern crate alloc;

use tokio::net::{TcpStream, UdpSocket};
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use tracing::*;

use std::net::SocketAddr;

use std::marker::{PhantomData, Sync};
use std::result::Result;
use std::sync::Arc;

use alloc::vec::Vec;
use postcard::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::msg::*;
use crate::Error;

use std::fmt::Debug;
/// Trait for Meadow-compatible data, requiring serde De\Serialize, Debug, and Clone
pub trait Message: Serialize + DeserializeOwned + Debug + Sync + Send + Clone {}
impl<T> Message for T where T: Serialize + DeserializeOwned + Debug + Sync + Send + Clone {}

/// State marker for a Node that has not been connected to a Host
#[derive(Debug)]
pub struct Idle;
/// State marker for a Node capable of manually sending publish/request messages
#[derive(Debug)]
pub struct Active;
/// State marker for a Node with an active topic subscription
#[derive(Debug)]
pub struct Subscription;

/// Composite data comprised of Meadow-compatible data and a String timestamp
#[derive(Debug, Clone)]
pub struct SubscriptionData<T: Message> {
    pub data: T,
    pub timestamp: String,
}

/// A named, strongly-typed Node capable of publish/request on Host
#[derive(Debug)]
pub struct Node<State, T: Message> {
    pub __state: PhantomData<State>,
    pub phantom: PhantomData<T>,
    pub cfg: NodeConfig<T>,
    pub runtime: Runtime,
    pub name: String,
    pub topic: String,
    pub stream: Option<TcpStream>,
    pub socket: Option<UdpSocket>,
    pub subscription_data: Arc<TokioMutex<Option<SubscriptionData<T>>>>,
    pub task_subscribe: Option<JoinHandle<()>>,
}

/// Attempts to create an async `TcpStream` connection with a Host at the specified socket address
pub async fn try_connection(host_addr: SocketAddr) -> Result<TcpStream, Error> {
    let mut connection_attempts = 0;
    let mut stream: Option<TcpStream> = None;
    while connection_attempts < 5 {
        match TcpStream::connect(host_addr).await {
            Ok(my_stream) => {
                stream = Some(my_stream);
                break;
            }
            Err(e) => {
                connection_attempts += 1;
                sleep(Duration::from_millis(1_000)).await;
                warn!("{:?}", e);
            }
        }
    }

    match stream {
        Some(stream) => Ok(stream),
        None => Err(Error::StreamConnection),
    }
}

/// Run the initial Node <=> Host connection handshake
pub async fn handshake(stream: TcpStream, topic: String) -> Result<TcpStream, Error> {
    loop {
        stream.writable().await.unwrap();
        match stream.try_write(topic.as_bytes()) {
            Ok(_n) => {
                info!("{}: Wrote {} bytes to host", topic, _n);
                break;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                } else {
                    error!("NODE handshake error: {:?}", e);
                }
            }
        }
    }
    info!("{}: Successfully connected to host", topic);
    // TO_DO: Is there a better way to do this?
    // Pause after connection to avoid accidentally including published data in initial handshake
    sleep(Duration::from_millis(20)).await;

    Ok(stream)
}

/// Send a `GenericMsg` of `MsgType` from the Node to the Host
pub async fn send_msg(stream: &mut &TcpStream, packet_as_bytes: Vec<u8>) -> Result<(), Error> {
    match stream.writable().await {
        Ok(_) => (),
        Err(_e) => return Err(Error::AccessStream),
    };

    // Write the request
    // TO_DO: This should be a loop with a maximum number of attempts
    loop {
        match stream.try_write(&packet_as_bytes) {
            Ok(_n) => {
                // info!("Node successfully wrote {}-byte request to host",n);
                break;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {}
                continue;
            }
        }
    }
    Ok(())
}

/// Set Node to wait for `GenericMsg` response from Host, with data to be deserialized into Node's <T>-type
pub async fn await_response<T: Message>(
    stream: &mut &TcpStream,
    max_buffer_size: usize,
) -> Result<GenericMsg, postcard::Error> {
    // Read the requested data into a buffer
    let mut buf = vec![0u8; max_buffer_size];
    loop {
        stream.readable().await.unwrap();
        match stream.try_read(&mut buf) {
            Ok(0) => continue,
            Ok(n) => {
                let bytes = &buf[..n];

                let msg: Result<GenericMsg, postcard::Error> = from_bytes(bytes);
                return msg;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {}
                continue;
            }
        }
    }
}
