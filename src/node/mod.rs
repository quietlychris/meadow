pub mod config;
pub mod network_config;
pub mod tcp;
pub mod udp;

#[cfg(feature = "quic")]
pub mod quic;

/// State marker for a Node that has not been connected to a Host
#[derive(Debug)]
pub struct Idle;
/// State marker for a Node capable of manually sending publish/request messages
#[derive(Debug)]
pub struct Active;
/// State marker for a Node with an active topic subscription
#[derive(Debug)]
pub struct Subscription;

mod private {
    pub trait Sealed {}

    use crate::node::network_config::{Tcp, Udp};
    impl Sealed for Udp {}
    impl Sealed for Tcp {}
    #[cfg(feature = "quic")]
    impl Sealed for crate::node::network_config::Quic {}

    use crate::node::{Active, Idle};
    impl Sealed for Idle {}
    impl Sealed for Active {}

    use crate::node::network_config::{Blocking, Nonblocking};
    impl Sealed for Blocking {}
    impl Sealed for Nonblocking {}
}

use tokio::io::AsyncWriteExt;
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

extern crate alloc;
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

use crate::node::config::NodeConfig;
use std::sync::Mutex;

/// Strongly-typed Node capable of publish/request on Host
#[derive(Debug)]
pub struct Node<B: Block, I: Interface + Default, State, T: Message> {
    pub(crate) __state: PhantomData<State>,
    pub(crate) __data_type: PhantomData<T>,
    pub(crate) cfg: NodeConfig<B, I, T>,
    pub(crate) runtime: Option<Runtime>,
    pub(crate) rt_handle: Option<Handle>,
    pub(crate) topic: String,
    pub(crate) stream: Option<TcpStream>,
    pub(crate) socket: Option<UdpSocket>,
    pub(crate) buffer: Arc<TokioMutex<Vec<u8>>>,
    #[cfg(feature = "quic")]
    pub(crate) endpoint: Option<Endpoint>,
    #[cfg(feature = "quic")]
    pub(crate) connection: Option<QuicConnection>,
    pub(crate) subscription_data: Arc<TokioMutex<Option<Msg<T>>>>,
    pub(crate) task_subscribe: Option<JoinHandle<()>>,
}

impl<B: Block, I: Interface + Default, State, T: Message> Node<B, I, State, T> {
    /// Get reference to the `Node`'s Tokio runtime if one exists
    pub fn runtime(&self) -> &Option<Runtime> {
        &self.runtime
    }

    /// Get reference to the `Node`'s runtime handle if one exists
    pub fn rt_handle(&self) -> &Option<Handle> {
        &self.rt_handle
    }

    /// Get `Node`'s configuration
    pub fn config(&self) -> &NodeConfig<B, I, T> {
        &self.cfg
    }

    /// Get `Node`'s topic
    pub fn topic(&self) -> String {
        self.topic.clone()
    }
}
