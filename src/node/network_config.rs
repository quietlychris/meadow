use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Configuration for network interfaces
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NetworkConfig {
    /// Socket address for the connected Host
    pub host_addr: SocketAddr,
    /// Max buffer size that the Node will allocate for Host responses
    pub max_buffer_size: usize,
}

impl NetworkConfig {
    /// Create a default config for specified network interface on port `25_000`
    pub fn default() -> Self {
        NetworkConfig {
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            max_buffer_size: 1024,
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

/// Strongly-typed alias of `NetworkConfig` for TCP configuration
pub use NetworkConfig as TcpConfig;
/// Strongly-typed alias of `NetworkConfig` for UDP configuration
pub use NetworkConfig as UdpConfig;
