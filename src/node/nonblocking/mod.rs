mod config;

#[cfg(feature = "quic")]
pub mod quic;
pub mod tcp;
pub mod udp;

pub use crate::node::network_config::NetworkConfig;
#[cfg(feature = "quic")]
pub use crate::node::network_config::Quic;
pub use crate::node::nonblocking::config::*;
#[cfg(feature = "quic")]
pub use crate::node::nonblocking::quic::*;
pub use crate::node::nonblocking::tcp::*;

extern crate alloc;

use tokio::net::{TcpStream, UdpSocket};
use tokio::runtime::{Handle, Runtime};
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

use crate::msg::*;
use crate::node::network_config::{Block, Interface};
use crate::Error;
use chrono::{DateTime, Utc};

// Quic stuff
#[cfg(feature = "quic")]
use quinn::Connection as QuicConnection;
#[cfg(feature = "quic")]
use quinn::{ClientConfig, Endpoint};
#[cfg(feature = "quic")]
use rustls::Certificate;
#[cfg(feature = "quic")]
use std::fs::File;
#[cfg(feature = "quic")]
use std::io::BufReader;

use std::sync::Mutex;

/// Strongly-typed Node capable of publish/request on Host
#[derive(Debug)]
pub struct Node<B: Block, I: Interface + Default, State, T: Message> {
    pub __state: PhantomData<State>,
    pub __data_type: PhantomData<T>,
    pub cfg: NodeConfig<B, I, T>,
    // pub runtime: Option<Runtime>,
    // pub rt_handle: Handle,
    pub topic: String,
    pub stream: Option<TcpStream>,
    pub socket: Option<UdpSocket>,
    pub buffer: Arc<TokioMutex<Vec<u8>>>,
    #[cfg(feature = "quic")]
    pub endpoint: Option<Endpoint>,
    #[cfg(feature = "quic")]
    pub connection: Option<QuicConnection>,
    pub subscription_data: Arc<TokioMutex<Option<Msg<T>>>>,
    pub task_subscribe: Option<JoinHandle<()>>,
}
