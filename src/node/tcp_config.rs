use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TcpConfig {
    pub host_addr: SocketAddr,
}

impl TcpConfig {
    /// Sets a default TCP host socket address of 127.0.0.1:25000
    pub fn default() -> Self {
        TcpConfig {
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
        }
    }
}
