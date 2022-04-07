use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UdpConfig {
    pub host_addr: SocketAddr,
}

impl UdpConfig {
    /// Sets a default UDP host socket address of 127.0.0.1:25000
    pub fn default() -> Self {
        UdpConfig {
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
        }
    }

    /// Create a configuration for a UdpSocket based on a socket address
    pub fn new(host_addr: impl Into<SocketAddr>) -> Self {
        UdpConfig {
            host_addr: host_addr.into(),
        }
    }
}
