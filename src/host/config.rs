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
    pub interface: String,
    pub socket_num: usize,
    pub store_filename: String,
    pub max_buffer_size: usize,
    pub max_name_size: usize,
}

impl HostConfig {
    /// Create a new HostConfig with all default options
    pub fn new(interface: impl Into<String>) -> HostConfig {
        HostConfig {
            interface: interface.into(),
            socket_num: 25_000,
            store_filename: "store".into(),
            max_buffer_size: 10_000,
            max_name_size: 100,
        }
    }

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

    /// Construct a Host based on the HostConfig's parameters
    pub fn build(self) -> Result<Host, Box<dyn Error>> {
        let ip = crate::get_ip(&self.interface)?;
        println!(
            "On interface {:?}, the device IP is: {:?}",
            &self.interface, &ip
        );

        let raw_addr = ip + ":" + &self.socket_num.to_string();
        // If the address won't parse, this should panic
        let _addr: SocketAddr = raw_addr.parse().unwrap_or_else(|_| {
            panic!(
                "The provided address st
        ring, \"{}\" is invalid",
                raw_addr
            )
        });

        let runtime = tokio::runtime::Runtime::new()?;

        let connections = Arc::new(StdMutex::new(Vec::new()));

        let config = sled::Config::default()
            .path(&self.store_filename)
            .temporary(true);
        let store: sled::Db = config.open()?;

        let reply_count = Arc::new(Mutex::new(0));

        Ok(Host {
            cfg: self,
            runtime,
            connections,
            task_listen: None,
            store: Some(store),
            reply_count,
        })
    }
}