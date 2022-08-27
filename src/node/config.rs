use tokio::sync::Mutex as TokioMutex;

use std::marker::PhantomData;
use std::result::Result;
use std::sync::Arc;

use std::fmt::Debug;

use crate::Error;
use crate::*;

/// Configuration of strongly-typed Node
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NodeConfig<T: Message> {
    pub name: String,
    pub topic: Option<String>,
    pub tcp: node::network_config::TcpConfig,
    pub udp: node::network_config::UdpConfig,
    pub phantom: PhantomData<T>,
}

impl<T: Message> NodeConfig<T> {
    /// Create a named, strongly-typed Node without an assigned topic
    pub fn new(name: impl Into<String>) -> NodeConfig<T> {
        NodeConfig {
            name: name.into(),
            topic: None,
            tcp: node::network_config::TcpConfig::default(),
            udp: node::network_config::UdpConfig::default(),
            phantom: PhantomData,
        }
    }

    /// Convenience method for re-setting the name of the Node to be generated
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Configure the UDP connection parameteres
    pub fn with_udp_config(mut self, udp_cfg: node::network_config::UdpConfig) -> Self {
        self.udp = udp_cfg;
        self
    }

    /// Configure the TCP connection parameteres
    pub fn with_tcp_config(mut self, tcp_cfg: node::network_config::TcpConfig) -> Self {
        self.tcp = tcp_cfg;
        self
    }

    /// Set topic of the generated Node
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Construct a Node from the specified configuration
    pub fn build(self) -> Result<Node<Idle, T>, Error> {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(_e) => return Err(Error::RuntimeCreation),
        };

        let topic = match &self.topic {
            Some(topic) => topic.to_owned(),
            None => panic!("Nodes must have an assigned topic to be built"),
        };

        Ok(Node::<Idle, T> {
            __state: PhantomData,
            phantom: PhantomData,
            cfg: self.clone(),
            runtime,
            stream: None,
            socket: None,
            name: self.name,
            topic,
            subscription_data: Arc::new(TokioMutex::new(None)),
            task_subscribe: None,
        })
    }
}
