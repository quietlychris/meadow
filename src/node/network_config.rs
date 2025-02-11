use std::fmt::Debug;
use std::marker::PhantomData;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

use crate::node::private;
pub trait Interface: private::Sealed + Default + Debug {}
pub trait Block: private::Sealed + Default + Sized + Debug {}

#[derive(Debug, Clone, Default)]
pub struct Tcp {}
impl Interface for Tcp {}
#[derive(Debug, Clone, Default)]
pub struct Udp {}
impl Interface for Udp {}

#[derive(Debug, Clone, Default)]
pub struct Quic {}
#[cfg(feature = "quic")]
impl Interface for Quic {}

#[derive(Debug, Clone, Default)]
pub struct Blocking;
impl Block for Blocking {}

#[derive(Debug, Clone, Default)]
pub struct Nonblocking;
impl Block for Nonblocking {}

/// Configuration for network interfaces
#[derive(Clone, Debug)]
pub struct NetworkConfig<Block, Interface>
where
    Interface: Default,
{
    __interface: PhantomData<Interface>,
    __block: PhantomData<Block>,
    /// Socket address for the connected Host
    pub host_addr: SocketAddr,
    /// Max buffer size that the Node will allocate for Host responses
    pub max_buffer_size: usize,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub send_tries: usize,
}

impl<B: Block> Default for NetworkConfig<B, Tcp> {
    fn default() -> NetworkConfig<B, Tcp> {
        Self {
            __interface: PhantomData::<Tcp>,
            __block: PhantomData::<B>,
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            max_buffer_size: 1024,
            cert_path: None,
            key_path: None,
            send_tries: 10,
        }
    }
}

impl<B: Block> NetworkConfig<B, Tcp> {
    /// Define a custom address for the Host to which the Node will connect
    pub fn set_host_addr(mut self, host_addr: impl Into<SocketAddr>) -> Self {
        self.host_addr = host_addr.into();
        self
    }

    /// Set a max buffer size for Host responses
    pub fn set_max_buffer_size(mut self, max_buffer_size: impl Into<usize>) -> Self {
        self.max_buffer_size = max_buffer_size.into();
        self
    }
}

impl<B: Block> Default for NetworkConfig<B, Udp> {
    fn default() -> NetworkConfig<B, Udp> {
        Self {
            __interface: PhantomData::<Udp>,
            __block: PhantomData::<B>,
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            max_buffer_size: 2048,
            cert_path: None,
            key_path: None,
            send_tries: 10,
        }
    }
}

impl<B: Block> NetworkConfig<B, Udp> {
    /// Define a custom address for the Host to which the Node will connect
    pub fn set_host_addr(mut self, host_addr: impl Into<SocketAddr>) -> Self {
        self.host_addr = host_addr.into();
        self
    }

    /// Set a max buffer size for Host responses
    pub fn set_max_buffer_size(mut self, max_buffer_size: impl Into<usize>) -> Self {
        self.max_buffer_size = max_buffer_size.into();
        self
    }
}

impl<B: Block> Default for NetworkConfig<B, Quic> {
    fn default() -> NetworkConfig<B, Quic> {
        Self {
            __interface: PhantomData::<Quic>,
            __block: PhantomData::<B>,
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            max_buffer_size: 4096,
            send_tries: 10,
            cert_path: Some(Path::new("target").join("cert.pem")),
            key_path: Some(Path::new("target").join("priv_key.pem")),
        }
    }
}

impl<B: Block> NetworkConfig<B, Quic> {
    /// Define a custom address for the Host to which the Node will connect
    pub fn set_host_addr(mut self, host_addr: impl Into<SocketAddr>) -> Self {
        self.host_addr = host_addr.into();
        self
    }

    /// Set a max buffer size for Host responses
    pub fn set_max_buffer_size(mut self, max_buffer_size: impl Into<usize>) -> Self {
        self.max_buffer_size = max_buffer_size.into();
        self
    }
}
