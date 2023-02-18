use std::fmt::Debug;
use std::marker::PhantomData;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

use crate::node::private;
pub trait Interface: private::Sealed + Default {}

#[derive(Debug)]
pub struct Tcp {}
#[derive(Debug)]
pub struct Udp {}

// Default implementations for both Udp and Tcp
impl Default for Udp {
    fn default() -> Self {
        Udp {}
    }
}
impl Interface for Udp {}


impl Default for Tcp {
    fn default() -> Self {
        Tcp {}
    }
}
impl Interface for Tcp {}

/// Configuration for network interfaces
#[derive(Clone, Debug)]
pub struct NetworkConfig<Interface>
where
    Interface: Default,
{
    __interface: PhantomData<Interface>,
    /// Socket address for the connected Host
    pub host_addr: SocketAddr,
    /// Max buffer size that the Node will allocate for Host responses
    pub max_buffer_size: usize,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
}

impl<I: Interface> Default for NetworkConfig<I> {
    fn default() -> NetworkConfig<I> {
        NetworkConfig {
            __interface: PhantomData::<I>,
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            max_buffer_size: 1024,
            cert_path: None,
            key_path: None
        }
    }
}

impl NetworkConfig<Tcp> {
    pub fn default() -> NetworkConfig<Tcp> {
        Self {
            __interface: PhantomData::<Tcp>,
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            max_buffer_size: 1024,
            cert_path: None,
            key_path: None,
        }
    }

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

impl NetworkConfig<Udp> {
    pub fn default() -> NetworkConfig<Udp> {
        Self {
            __interface: PhantomData::<Udp>,
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            max_buffer_size: 2048,
            cert_path: None,
            key_path: None,
        }
    }

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

/*
impl<I: Interface> NetworkConfig<I> {

    pub fn default() -> NetworkConfig<I> {
        Self {
            __interface: PhantomData::<I>,
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            max_buffer_size: 1024,
            //cert_path: None,
            //key_path: None,
        }
    }

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
*/

/*
impl NetworkConfig<Tcp> {
    /// Create a default config for specified network interface on port `25_000`
    pub fn default() -> Self {
        NetworkConfig {
            __interface: PhantomData,
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            max_buffer_size: 1024,
            cert_path: None,
            key_path: None,
        }
    }

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

impl NetworkConfig<Udp> {
    /// Create a default config for specified network interface on port `25_000`
    pub fn default() -> Self {
        NetworkConfig {
            __interface: PhantomData,
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            max_buffer_size: 1024,
            cert_path: None,
            key_path: None,
        }
    }

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

impl NetworkConfig<Quic> {
    /// Create a default config for specified network interface on port `25_000`
    pub fn default() -> Self {
        NetworkConfig {
            __interface: PhantomData,
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            max_buffer_size: 1024,
            cert_path: Some(Path::new("target").join("cert.pem")),
            key_path: Some(Path::new("target").join("priv_key.pem")),
        }
    }

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
*/

/*
/// Strongly-typed alias of `NetworkConfig` for TCP configuration
pub use NetworkConfig as TcpConfig;
/// Strongly-typed alias of `NetworkConfig` for UDP configuration
pub use NetworkConfig as UdpConfig;

/// Configuration type containing `NetworkConfig` and certification path information
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NetworkConfig<Quic> {
    pub network_cfg: NetworkConfig,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

impl QuicConfig {
    pub fn default() -> Self {
        QuicConfig {
            network_cfg: NetworkConfig::default(),
            cert_path: Path::new("target").join("cert.pem"),
            key_path: Path::new("target").join("priv_key.pem"),
        }
    }
}
*/
