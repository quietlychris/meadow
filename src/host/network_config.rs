use std::path::{Path, PathBuf};

/// Configuration for network interfaces
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NetworkConfig {
    pub interface: String,
    pub socket_num: u16,
    pub max_buffer_size: usize,
    pub max_name_size: usize,
}

impl NetworkConfig {
    // Create a default config for specified network interface on port `25_000`
    pub fn default(interface: impl Into<String>) -> Self {
        NetworkConfig {
            interface: interface.into(),
            socket_num: 25_000,
            max_buffer_size: 10_000,
            max_name_size: 100,
        }
    }

    /// Set the socket number on the network port
    pub fn set_socket_num(mut self, socket_num: impl Into<u16>) -> NetworkConfig {
        self.socket_num = socket_num.into();
        self
    }

    /// Set the maximum buffer size for packets intended to be received
    pub fn set_max_buffer_size(mut self, max_buffer_size: usize) -> NetworkConfig {
        self.max_buffer_size = max_buffer_size;
        self
    }

    /// Set the maximum buffer size for the name of each topic
    pub fn set_max_name_size(mut self, max_name_size: usize) -> NetworkConfig {
        self.max_name_size = max_name_size;
        self
    }
}

/// Strongly-typed alias of `NetworkConfig` for TCP configuration
pub use NetworkConfig as TcpConfig;
/// Strongly-typed alias of `NetworkConfig` for UDP configuration
pub use NetworkConfig as UdpConfig;

/// Configuration type containing `NetworkConfig` and certification path information
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QuicConfig {
    pub network_cfg: NetworkConfig,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

impl Default for QuicConfig {
    fn default() -> Self {
        QuicConfig {
            network_cfg: NetworkConfig {
                interface: "lo".into(),
                socket_num: 25_000,
                max_buffer_size: 10_000,
                max_name_size: 100,
            },
            cert_path: Path::new("target").join("cert.pem"),
            key_path: Path::new("target").join("priv_key.pem"),
        }
    }
}

impl QuicConfig {
    pub fn new(interface: impl Into<String>) -> Self {
        QuicConfig {
            network_cfg: NetworkConfig {
                interface: interface.into(),
                socket_num: 25_000,
                max_buffer_size: 10_000,
                max_name_size: 100,
            },
            cert_path: Path::new("target").join("cert.pem"),
            key_path: Path::new("target").join("priv_key.pem"),
        }
    }
}
