use crate::Error;
use std::result::Result;
use std::sync::{Mutex, Arc};

use crate::node::network_config::*;
use crate::node::blocking::{Active, Idle, Message, Node};
use crate::node::blocking::{Interface, NetworkConfig};
use std::default::Default;
use std::marker::PhantomData;

/// Configuration of strongly-typed Node
#[derive(Debug, Clone)]
pub struct NodeConfig<I: Interface + Default, T: Message> {
    pub __data_type: PhantomData<T>,
    pub topic: Option<String>,
    pub network_cfg: NetworkConfig<I>,
}

impl<I: Interface + Default + Clone, T: Message> NodeConfig<I, T>
where
    NetworkConfig<I>: Default,
{
    /// Create a named, strongly-typed Node without an assigned topic
    pub fn new(topic: impl Into<String>) -> NodeConfig<I, T> {
        NodeConfig {
            __data_type: PhantomData,
            topic: Some(topic.into()),
            network_cfg: NetworkConfig::<I>::default(),
        }
    }

    /// Configure the TCP connection parameteres
    pub fn with_config(mut self, network_cfg: NetworkConfig<I>) -> Self {
        self.network_cfg = network_cfg;
        self
    }

    /// Construct a Node from the specified configuration
    pub fn build(self) -> Result<Node<I, Idle, T>, Error> {

        let topic = match &self.topic {
            Some(topic) => topic.to_owned(),
            None => panic!("Nodes must have an assigned topic to be built"),
        };

        Ok(Node::<I, Idle, T> {
            __state: PhantomData::<Idle>,
            __data_type: PhantomData::<T>,
            cfg: self,
            stream: None,
            socket: None,
            #[cfg(feature = "quic")]
            endpoint: None,
            #[cfg(feature = "quic")]
            connection: None,
            topic,
            subscription_data: Arc::new(Mutex::new(None)),
            task_subscribe: None,
        })
    }
}
