use crate::*;

use tokio::sync::Mutex as TokioMutex;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use std::error::Error;
use std::marker::PhantomData;
use std::result::Result;
use std::sync::Arc;

use std::fmt::Debug;

/// Configuration of strongly-typed Node
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NodeConfig<T: Message> {
    pub host_addr: SocketAddr,
    pub name: String,
    pub topic: Option<String>,
    pub phantom: PhantomData<T>,
}

impl<T: Message> NodeConfig<T> {
    /// Create a named, strongly-typed Node without an assigned topic
    pub fn new(name: impl Into<String>) -> NodeConfig<T> {
        NodeConfig {
            name: name.into(),
            topic: None,
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            phantom: PhantomData,
        }
    }

    /// Convenience method for re-setting the name of the Node to be generated
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set topic of the generated Node
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Assign an address for the Host the Node will attempt to connect with
    pub fn host_addr(mut self, host_addr: impl Into<SocketAddr>) -> Self {
        self.host_addr = host_addr.into();
        self
    }

    /// Construct a Node from the specified configuration
    pub fn build(self) -> Result<Node<Idle, T>, Box<dyn Error>> {
        let runtime = tokio::runtime::Runtime::new()?;

        let topic = match self.topic {
            Some(topic) => topic,
            None => panic!("Nodes must have an assigned topic to be built"),
        };

        Ok(Node::<Idle, T> {
            __state: PhantomData,
            phantom: PhantomData,
            runtime,
            stream: None,
            host_addr: self.host_addr,
            name: self.name,
            topic,
            subscription_data: Arc::new(TokioMutex::new(None)),
            task_subscribe: None,
        })
    }
}
