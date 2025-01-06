use crate::*;
use chrono::Utc;

// Tokio for async
use tokio::sync::Mutex; // as TokioMutex;
                        // Multi-threading primitives
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
// Misc other imports
use crate::prelude::*;
use std::result::Result;

#[doc(hidden)]
pub use sled::Config as SledConfig;

/// Host configuration structure
#[derive(Debug)]
pub struct HostConfig {
    pub sled_cfg: sled::Config,
    pub tcp_cfg: Option<host::TcpConfig>,
    pub udp_cfg: Option<host::UdpConfig>,
    #[cfg(feature = "quic")]
    pub quic_cfg: Option<host::QuicConfig>,
}

impl Default for HostConfig {
    /// Create a new `HostConfig` with all default options
    fn default() -> HostConfig {
        // Default sled database configuration
        let date = Utc::now();
        let stamp = format!(
            "{}_{}_UTC",
            date.date_naive(),
            date.time().format("%H:%M:%S")
        );
        let sled_cfg = sled::Config::default()
            .path(format!("./logs/{}.sled", stamp))
            .temporary(true);

        #[cfg(feature = "quic")]
        {
            return HostConfig {
                sled_cfg,
                tcp_cfg: Some(host::TcpConfig::default("lo")),
                udp_cfg: None,
                quic_cfg: Some(host::QuicConfig::default()),
            };
        }
        #[cfg(not(feature = "quic"))]
        {
            return HostConfig {
                sled_cfg,
                tcp_cfg: Some(host::TcpConfig::default("lo")),
                udp_cfg: Some(host::UdpConfig::default("lo")),
            };
        }
    }
}

impl HostConfig {
    /// Add the Sled database configuration to the Host configuration
    pub fn with_sled_config(mut self, sled_cfg: sled::Config) -> HostConfig {
        self.sled_cfg = sled_cfg;
        self
    }

    /// Assign a configuration to the Host `TcpListener`
    pub fn with_tcp_config(mut self, tcp_cfg: Option<host::TcpConfig>) -> HostConfig {
        self.tcp_cfg = tcp_cfg;
        self
    }

    /// Assign a configuration to the Host `UdpSocket`
    pub fn with_udp_config(mut self, udp_cfg: Option<host::UdpConfig>) -> HostConfig {
        self.udp_cfg = udp_cfg;
        self
    }

    /// Assign a configuration to the Host's QUIC `Endpoint` server
    #[cfg(feature = "quic")]
    pub fn with_quic_config(mut self, quic_cfg: Option<host::QuicConfig>) -> HostConfig {
        self.quic_cfg = quic_cfg;
        self
    }

    /// Construct a Host based on the `HostConfig`'s parameters
    pub fn build(self) -> Result<Host, Error> {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(_e) => return Err(Error::RuntimeCreation),
        };

        let connections = Arc::new(StdMutex::new(Vec::new()));
        let store: sled::Db = self.sled_cfg.open()?;

        Ok(Host {
            cfg: self,
            runtime,
            task_listen_tcp: None,
            connections,
            task_listen_udp: None,
            #[cfg(feature = "quic")]
            task_listen_quic: None,
            store,
        })
    }
}
