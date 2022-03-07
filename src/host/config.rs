use crate::*;

// Tokio for async
use tokio::sync::Mutex; // as TokioMutex;
                        // Multi-threading primitives
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
// Misc other imports
use std::error::Error;
use std::net::SocketAddr;
use std::result::Result;

/// Host configuration structure
#[derive(Debug)]
pub struct HostConfig {
    pub sled_cfg: sled::Config,
    pub tcp_cfg: TcpConfig,
    pub udp_cfg: UdpConfig,
}

impl HostConfig {
    /// Create a new HostConfig with all default options
    pub fn default() -> HostConfig {
        // Default sled database configuration
        let sled_cfg = sled::Config::default().path("store").temporary(true);

        HostConfig {
            sled_cfg,
            tcp_cfg: TcpConfig::default("lo"),
            udp_cfg: UdpConfig::default("lo"),
        }
    }

    ///
    pub fn with_sled_config(mut self, sled_cfg: sled::Config) -> HostConfig {
        self.sled_cfg = sled_cfg;
        self
    }

    pub fn with_tcp_config(mut self, tcp_cfg: TcpConfig) -> HostConfig {
        self.tcp_cfg = tcp_cfg;
        self
    }

    pub fn with_udp_config(mut self, udp_cfg: UdpConfig) -> HostConfig {
        self.udp_cfg = udp_cfg;
        self
    }

    /*
    /// Assign a particular socket number for the Host's TcpListener
    pub fn socket_num(mut self, socket_num: usize) -> HostConfig {
        self.socket_num = socket_num;
        self
    }

    /// Change the maximum size of the buffer space allocated for received messages
    pub fn max_buffer_size(mut self, max_buffer_size: impl Into<usize>) -> HostConfig {
        self.max_buffer_size = max_buffer_size.into();
        self
    }

    /// Change the maximum size of the buffer space allocated for Node names
    pub fn max_name_size(mut self, max_name_size: impl Into<usize>) -> HostConfig {
        self.max_buffer_size = max_name_size.into();
        self
    }

    /// Change the filename of the Host's sled key-value store
    pub fn store_filename(mut self, store_filename: impl Into<String>) -> HostConfig {
        self.store_filename = store_filename.into();
        self
    }
    */

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
