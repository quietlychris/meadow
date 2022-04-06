use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TcpConfig {
    pub host_addr: SocketAddr,
    pub max_buffer_size: usize,
}

impl TcpConfig {
    /// Sets a default TCP host socket address of 127.0.0.1:25000
    pub fn default() -> Self {
        TcpConfig {
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            max_buffer_size: 10_000,
        }
    }

    /// Set the maximum buffer size in bytes for packets over TCP
    pub fn set_max_buffer_size(mut self, max_buffer_size: usize) -> Self {
        self.max_buffer_size = max_buffer_size;
        self
    }

    /// Assign a TCP address for the Host the Node will attempt to connect with
    pub fn set_host_addr(mut self, host_addr: impl Into<SocketAddr>) -> Self {
        self.host_addr = host_addr.into();
        self
    }
}
