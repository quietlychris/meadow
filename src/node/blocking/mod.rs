mod config;
// #[cfg(feature = "quic")]
//mod quic;
//pub mod tcp;
//pub mod udp;

use std::thread::JoinHandle;
use std::net::{UdpSocket, TcpStream};

pub use crate::node::blocking::config::*;
pub use crate::node::network_config::NetworkConfig;
#[cfg(feature = "quic")]
pub use crate::node::network_config::Quic;
pub use crate::node::network_config::{Tcp, Udp};
//#[cfg(feature = "quic")]
//pub use crate::node::nonblocking::quic::*;
pub use crate::node::tcp::*;

extern crate alloc;

use tracing::*;

use std::net::SocketAddr;

use std::marker::{PhantomData, Sync};
use std::result::Result;
use std::sync::{Mutex, Arc};

use alloc::vec::Vec;
use postcard::*;

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

/// State marker for a Node that has not been connected to a Host
#[derive(Debug)]
pub struct Idle;
/// State marker for a Node capable of manually sending publish/request messages
#[derive(Debug)]
pub struct Active;
/// State marker for a Node with an active topic subscription
#[derive(Debug)]
pub struct Subscription;

/// A named, strongly-typed Node capable of publish/request on Host
#[derive(Debug)]
pub struct Node<I: Interface + Default, State, T: Message> {
    pub __state: PhantomData<State>,
    pub __data_type: PhantomData<T>,
    pub cfg: NodeConfig<I, T>,
    pub topic: String,
    pub stream: Option<TcpStream>,
    pub socket: Option<UdpSocket>,
    #[cfg(feature = "quic")]
    pub endpoint: Option<Endpoint>,
    #[cfg(feature = "quic")]
    pub connection: Option<QuicConnection>,
    pub subscription_data: Arc<Mutex<Option<Msg<T>>>>,
    pub task_subscribe: Option<JoinHandle<()>>,
}
