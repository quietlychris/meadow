use crate::*;

// Tokio for async
use tokio::sync::Mutex; // as TokioMutex;
                        // Multi-threading primitives
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
// Misc other imports
use std::error::Error;
use std::result::Result;

/// Host configuration structure
#[derive(Debug)]
pub struct HostConfig {
    pub sled_cfg: sled::Config,
    pub tcp_cfg: Option<TcpConfig>,
    pub udp_cfg: Option<UdpConfig>,
}

impl HostConfig {
    /// Create a new HostConfig with all default options
    pub fn default() -> HostConfig {
        // Default sled database configuration
        let sled_cfg = sled::Config::default().path("store").temporary(true);

        HostConfig {
            sled_cfg,
            tcp_cfg: Some(TcpConfig::default("lo")),
            udp_cfg: Some(UdpConfig::default("lo")),
        }
    }

    /// Add the Sled database configuration to the Host configuration
    pub fn with_sled_config(mut self, sled_cfg: sled::Config) -> HostConfig {
        self.sled_cfg = sled_cfg;
        self
    }

    ///
    pub fn with_tcp_config(mut self, tcp_cfg: Option<TcpConfig>) -> HostConfig {
        self.tcp_cfg = tcp_cfg;
        self
    }

    pub fn with_udp_config(mut self, udp_cfg: Option<UdpConfig>) -> HostConfig {
        self.udp_cfg = udp_cfg;
        self
    }

    /// Construct a Host based on the HostConfig's parameters
    pub fn build(self) -> Result<Host, Box<dyn Error>> {
        /*
        let ip = crate::get_ip(&self.interface)?;
        println!(
            "On interface {:?}, the device IP is: {:?}",
            &self.interface, &ip
        );

        let raw_addr = ip + ":" + &self.socket_num.to_string();
        // If the address won't parse, this should panic
        let _addr: SocketAddr = raw_addr
            .parse()
            .unwrap_or_else(|_| panic!("The provided address string, \"{}\" is invalid", raw_addr));
        */

        let runtime = tokio::runtime::Runtime::new()?;

        let connections = Arc::new(StdMutex::new(Vec::new()));
        let store: sled::Db = self.sled_cfg.open()?;

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
