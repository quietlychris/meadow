use crate::{Message, Error};
use std::result::Result;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::Mutex as TokioMutex;

use crate::node::network_config::*;
use crate::node::Node;
use crate::node::{Active, Idle};
use std::default::Default;
use std::marker::PhantomData;
use std::sync::Mutex;

/// Defines whether the Node should own it's async runtime or use a provided handle to an external one
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub owned_runtime: bool,
    pub rt_handle: Option<Handle>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        RuntimeConfig {
            owned_runtime: true,
            rt_handle: None,
        }
    }
}

impl RuntimeConfig {
    /// Set whether the Node should own its own runtime
    pub fn with_owned_runtime(mut self, owned_runtime: bool) -> Self {
        self.owned_runtime = owned_runtime;
        self
    }

    /// Supply an external runtime handle
    pub fn with_rt_handle(mut self, rt_handle: Option<Handle>) -> Self {
        self.rt_handle = rt_handle;
        self
    }
}

/// Configuration of strongly-typed Node
#[derive(Debug, Clone)]
pub struct NodeConfig<B: Block, I: Interface + Default, T: Message> {
    pub __data_type: PhantomData<T>,
    pub topic: Option<String>,
    pub network_cfg: NetworkConfig<B, I>,
    pub runtime_cfg: RuntimeConfig,
}

impl<B: Block, I: Interface + Default + Clone, T: Message> NodeConfig<B, I, T>
where
    NetworkConfig<B, I>: Default,
{
    /// Create a named, strongly-typed Node without an assigned topic
    pub fn new(topic: impl Into<String>) -> NodeConfig<B, I, T> {
        NodeConfig {
            __data_type: PhantomData,
            topic: Some(topic.into()),
            network_cfg: NetworkConfig::<B, I>::default(),
            runtime_cfg: RuntimeConfig::default(),
        }
    }

    /// Configure the TCP connection parameteres
    pub fn with_config(mut self, network_cfg: NetworkConfig<B, I>) -> Self {
        self.network_cfg = network_cfg;
        self
    }

    pub fn with_runtime_config(mut self, runtime_cfg: RuntimeConfig) -> Self {
        self.runtime_cfg = runtime_cfg;
        self
    }

    /// Construct a Node from the specified configuration
    pub fn build(self) -> Result<Node<B, I, Idle, T>, Error> {
        /*         let (runtime, rt_handle) = {
            if self.runtime_cfg.owned_runtime {
                let runtime = match tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(_e) => return Err(Error::RuntimeCreation),
                };
                let handle = runtime.handle().clone();
                (Some(runtime), handle)
            } else if let Some(rt_handle) = self.runtime_cfg.rt_handle.clone() {
                (None, rt_handle)
            } else {
                return Err(Error::RuntimeCreation);
            }
        }; */

        let topic = match &self.topic {
            Some(topic) => topic.to_owned(),
            None => panic!("Nodes must have an assigned topic to be built"),
        };

        let max_buffer_size = self.network_cfg.max_buffer_size;

        Ok(Node::<B, I, Idle, T> {
            __state: PhantomData::<Idle>,
            __data_type: PhantomData::<T>,
            cfg: self,
            stream: None,
            socket: None,
            buffer: Arc::new(TokioMutex::new(vec![0u8; max_buffer_size])),
            //buffer: Arc::new(Vec::with_capacity(max_buffer_size)),
            #[cfg(feature = "quic")]
            endpoint: None,
            #[cfg(feature = "quic")]
            connection: None,
            topic,
            subscription_data: Arc::new(TokioMutex::new(None)),
            task_subscribe: None,
        })
    }
}
