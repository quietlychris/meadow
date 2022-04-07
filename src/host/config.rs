use crate::*;

// Tokio for async
use tokio::sync::Mutex; // as TokioMutex;
                        // Multi-threading primitives
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
// Misc other imports
use crate::Error;
use std::result::Result;

/// Host configuration structure
#[derive(Debug)]
pub struct HostConfig {
    pub sled_cfg: sled::Config,
    pub tcp_cfg: Option<host::TcpConfig>,
    pub udp_cfg: Option<host::UdpConfig>,
}

impl HostConfig {
    /// Create a new HostConfig with all default options
    pub fn default() -> HostConfig {
        // Default sled database configuration
        let sled_cfg = sled::Config::default().path("store").temporary(true);

        HostConfig {
            sled_cfg,
            tcp_cfg: Some(host::TcpConfig::default("lo")),
            udp_cfg: Some(host::UdpConfig::default("lo")),
        }
    }

    /// Add the Sled database configuration to the Host configuration
    pub fn with_sled_config(mut self, sled_cfg: sled::Config) -> HostConfig {
        self.sled_cfg = sled_cfg;
        self
    }

    /// Assign a configuration to the Host TcpListener
    pub fn with_tcp_config(mut self, tcp_cfg: Option<host::TcpConfig>) -> HostConfig {
        self.tcp_cfg = tcp_cfg;
        self
    }

    /// Assign a configuration to the Host UdpSocket
    pub fn with_udp_config(mut self, udp_cfg: Option<host::UdpConfig>) -> HostConfig {
        self.udp_cfg = udp_cfg;
        self
    }

    /// Construct a Host based on the HostConfig's parameters
    pub fn build(self) -> Result<Host, Error> {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(_e) => return Err(Error::RuntimeCreation),
        };

        let connections = Arc::new(StdMutex::new(Vec::new()));
        let store: sled::Db = match self.sled_cfg.open() {
            Ok(store) => store,
            Err(_e) => return Err(Error::OpeningSled),
        };

        let reply_count = Arc::new(Mutex::new(0));

        Ok(Host {
            cfg: self,
            runtime,
            connections,
            task_listen_tcp: None,
            task_listen_udp: None,
            store: Some(store),
            reply_count,
        })
    }
}
