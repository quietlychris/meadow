use std::fmt::Debug;
use std::marker::PhantomData;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

use crate::NetworkConfig;

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