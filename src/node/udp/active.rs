use crate::node::Interface;
use crate::Error;
use crate::*;
use std::marker::PhantomData;

use chrono::Utc;

use postcard::*;
use quinn::Connection as QuicConnection;
use std::result::Result;
use tracing::*;

/// Udp implements the Interface trait
impl Interface for Udp {}

impl<T: Message> From<Node<Udp, Idle, T>> for Node<Udp, Active, T> {
    fn from(node: Node<Udp, Idle, T>) -> Self {
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

impl<T: Message + 'static> Node<Udp, Active, T> {
    #[tracing::instrument]
    pub fn publish(&self, val: T) -> Result<(), Error> {
        let val_vec: Vec<u8> = match to_allocvec(&val) {
            Ok(val_vec) => val_vec,
            Err(_e) => return Err(Error::Serialization),
        };

        let packet = GenericMsg {
            msg_type: MsgType::SET,
            timestamp: Utc::now(),
            name: self.name.to_string(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: val_vec.to_vec(),
        };

        let packet_as_bytes: Vec<u8> = match to_allocvec(&packet) {
            Ok(packet) => packet,
            Err(_e) => return Err(Error::Serialization),
        };

        let socket = match self.socket.as_ref() {
            Some(socket) => socket,
            None => return Err(Error::AccessSocket),
        };

        self.runtime.block_on(async {
            match socket
                .send_to(&packet_as_bytes, self.cfg.network_cfg.host_addr)
                .await
            {
                Ok(_len) => Ok(()),
                Err(e) => {
                    error!("{:?}", e);
                    Err(Error::UdpSend)
                }
            }
        })
    }
}
