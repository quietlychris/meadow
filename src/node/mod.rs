mod config;
mod network_config;
#[cfg(feature = "quic")]
mod quic;
mod tcp;
mod udp;

pub use crate::node::config::*;
pub use crate::node::network_config::NetworkConfig;
#[cfg(feature = "quic")]
pub use crate::node::network_config::Quic;
pub use crate::node::network_config::{Tcp, Udp};
#[cfg(feature = "quic")]
pub use crate::node::quic::*;
pub use crate::node::tcp::*;
pub use crate::node::udp::*;

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
use crate::node::network_config::Interface;
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
    pub timestamp: DateTime<Utc>,
}

mod private {
    pub trait Sealed {}
    impl Sealed for crate::Udp {}
    impl Sealed for crate::Tcp {}
    #[cfg(feature = "quic")]
    impl Sealed for crate::Quic {}

    impl Sealed for crate::Idle {}
    impl Sealed for crate::Active {}
}

/// A named, strongly-typed Node capable of publish/request on Host
#[derive(Debug)]
pub struct Node<I: Interface + Default, State, T: Message> {
    pub __state: PhantomData<State>,
    pub __data_type: PhantomData<T>,
    pub cfg: NodeConfig<I, T>,
    pub runtime: Runtime,
    pub name: String,
    pub topic: String,
    pub stream: Option<TcpStream>,
    pub socket: Option<UdpSocket>,
    #[cfg(feature = "quic")]
    pub endpoint: Option<Endpoint>,
    #[cfg(feature = "quic")]
    pub connection: Option<QuicConnection>,
    pub subscription_data: Arc<TokioMutex<Option<SubscriptionData<T>>>>,
    pub task_subscribe: Option<JoinHandle<()>>,
}
