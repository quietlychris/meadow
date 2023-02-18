extern crate alloc;
use crate::Error;
use crate::*;

use crate::node::*;

use tokio::net::UdpSocket;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

use tracing::*;

use std::net::SocketAddr;
use std::result::Result;
use std::sync::Arc;

use alloc::vec::Vec;
use postcard::*;
use std::marker::PhantomData;

// Quic
use quinn::Endpoint;

use crate::msg::*;
use crate::node::quic::generate_client_config_from_certs;
use chrono::Utc;

impl<T: Message> From<Node<Quic, Idle, T>> for Node<Quic, Active, T> {
    fn from(node: Node<Quic, Idle, T>) -> Self {
        Self {
            //__interface: PhantomData,
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
            stream: node.stream,
            name: node.name,
            topic: node.topic,
            socket: node.socket,
            endpoint: node.endpoint,
            subscription_data: node.subscription_data,
            task_subscribe: None,
        }
    }
}

impl<T: Message + 'static> Node<Quic, Idle, T> {
    /// Attempt connection from the Node to the Host located at the specified address
    //#[tracing::instrument(skip_all)]
    pub fn activate(mut self) -> Result<Node<Quic, Active, T>, Error> {
        info!("Attempting QUIC connection");
        let endpoint = self.runtime.block_on(async move {
            // QUIC, needs to be done inside of a tokio context
            let client_cfg = generate_client_config_from_certs();
            let client_addr = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
            let mut endpoint = Endpoint::client(client_addr).unwrap();
            endpoint.set_default_client_config(client_cfg);
            endpoint
        });
        info!("{:?}", &endpoint.local_addr());
        self.endpoint = Some(endpoint);

        Ok(Node::<Quic, Active, T>::from(self))
    }
}
