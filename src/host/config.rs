use crate::*;
use chrono::Utc;

#[cfg(feature = "redb")]
use redb::Database;
// Tokio for async
use std::path::PathBuf;
use tokio::sync::Mutex;
// as TokioMutex;
// Multi-threading primitives
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
// Misc other imports
use crate::prelude::*;
use std::result::Result;

#[doc(hidden)]
pub use sled::Config as SledConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    path: String,
}

/// Host configuration structure
#[derive(Debug)]
pub struct HostConfig {
    #[cfg(not(feature = "redb"))]
    pub sled_cfg: sled::Config,
    storage_cfg: StorageConfig,
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

        let storage_cfg = StorageConfig {
            #[cfg(feature = "redb")]
            path: format!("./logs/{}.redb", stamp),
            #[cfg(not(feature = "redb"))]
            path: format!("./logs/{}.sled", stamp),
        };

        let sled_cfg = sled::Config::default()
            .path(format!("./logs/{}.sled", stamp))
            .temporary(true);

        #[cfg(feature = "quic")]
        {
            return HostConfig {
                #[cfg(not(feature = "redb"))]
                sled_cfg,
                storage_cfg,
                tcp_cfg: Some(host::TcpConfig::default("lo")),
                udp_cfg: None,
                quic_cfg: Some(host::QuicConfig::default()),
            };
        }
        #[cfg(not(feature = "quic"))]
        {
            return HostConfig {
                #[cfg(not(feature = "redb"))]
                sled_cfg,
                storage_cfg,
                tcp_cfg: Some(host::TcpConfig::default("lo")),
                udp_cfg: Some(host::UdpConfig::default("lo")),
            };
        }
    }
}

impl HostConfig {
    #[cfg(not(feature = "redb"))]
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
        #[cfg(not(feature = "redb"))]
        let store: sled::Db = self.sled_cfg.open()?;
        #[cfg(feature = "redb")]
        let store = {
            let db = Database::create(&self.storage_cfg.path)
                .map_err(|e| crate::error::redb::RedbError::DatabaseError)?;
            Arc::new(db)
        };

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
