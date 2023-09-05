use crate::node::nonblocking::network_config::Udp;
use crate::node::nonblocking::Interface;
use crate::node::Node;
use crate::Error;
use crate::{Active, Idle, MsgType};
use std::marker::PhantomData;

use crate::node::udp::*;

use chrono::Utc;

use postcard::*;
#[cfg(feature = "quic")]
use quinn::Connection as QuicConnection;
use std::result::Result;
use tracing::*;

/// Udp implements the Interface trait
impl Interface for Udp {}

impl<T: Message> From<Node<Udp, Idle, T>> for Node<Udp, Active, T> {
    fn from(node: Node<Udp, Idle, T>) -> Self {
        Self {
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
            stream: node.stream,
            topic: node.topic,
            socket: node.socket,
            #[cfg(feature = "quic")]
            endpoint: node.endpoint,
            #[cfg(feature = "quic")]
            connection: node.connection,
            subscription_data: node.subscription_data,
            task_subscribe: None,
        }
    }
}

impl<T: Message + 'static> Node<Udp, Active, T> {
    #[tracing::instrument]
    pub fn publish(&self, val: T) -> Result<(), Error> {
        let data: Vec<u8> = match to_allocvec(&val) {
            Ok(data) => data,
            Err(_e) => return Err(Error::Serialization),
        };

        let generic = GenericMsg {
            msg_type: MsgType::SET,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data,
        };

        let packet_as_bytes: Vec<u8> = match to_allocvec(&generic) {
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

    #[tracing::instrument]
    pub fn request(&self) -> Result<Msg<T>, Error> {
        let socket = match self.socket.as_ref() {
            Some(socket) => socket,
            None => return Err(Error::AccessSocket),
        };

        let packet: GenericMsg = GenericMsg {
            msg_type: MsgType::GET,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: Vec::new(),
        };

        let packet_as_bytes: Vec<u8> = match to_allocvec(&packet) {
            Ok(packet) => packet,
            Err(_e) => return Err(Error::Serialization),
        };

        self.runtime.block_on(async {
            if let Ok(_n) = self.send_msg(packet_as_bytes).await {
                match await_response::<T>(socket, self.cfg.network_cfg.max_buffer_size).await {
                    Ok(msg) => Ok(msg),
                    Err(_e) => Err(Error::Deserialization),
                }
            } else {
                Err(Error::BadResponse)
            }
        })
    }

    async fn send_msg(&self, packet_as_bytes: Vec<u8>) -> Result<usize, Error> {
        match &self.socket {
            Some(socket) => {
                match socket.writable().await {
                    Ok(_) => (),
                    Err(_e) => return Err(Error::AccessSocket),
                };

                // Write the request
                for _ in 0..10 {
                    match socket
                        .send_to(&packet_as_bytes, self.cfg.network_cfg.host_addr)
                        .await
                    {
                        Ok(n) => return Ok(n),
                        Err(_e) => {}
                    }
                }
                Err(Error::BadResponse)
            }
            None => Err(Error::AccessSocket),
        }
    }
}
