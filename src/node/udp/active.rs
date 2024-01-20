use crate::node::network_config::Udp;
use crate::node::Interface;
use crate::node::Node;
use crate::Error;
use crate::{Active, Idle, MsgType};
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

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
            rt_handle: node.rt_handle,
            stream: node.stream,
            topic: node.topic,
            socket: node.socket,
            buffer: node.buffer,
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
    #[inline]
    pub fn publish(&self, val: T) -> Result<(), Error> {
        let data: Vec<u8> = to_allocvec(&val)?;

        let generic = GenericMsg {
            msg_type: MsgType::SET,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data,
        };

        let packet_as_bytes: Vec<u8> = to_allocvec(&generic)?;

        let socket = match self.socket.as_ref() {
            Some(socket) => socket,
            None => return Err(Error::AccessSocket),
        };

        self.rt_handle.block_on(async {
            socket
                .send_to(&packet_as_bytes, self.cfg.network_cfg.host_addr)
                .await?;
            Ok(())
        })
    }

    #[tracing::instrument]
    #[inline]
    pub fn request(&self) -> Result<Msg<T>, Error> {
        let packet: GenericMsg = GenericMsg {
            msg_type: MsgType::GET,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: Vec::new(),
        };

        let packet_as_bytes: Vec<u8> = to_allocvec(&packet)?;
        let buffer = self.buffer.clone();

        self.rt_handle.block_on(async {
            if let Some(socket) = &self.socket {
                send_msg(socket, packet_as_bytes, self.cfg.network_cfg.host_addr).await?;
                let msg = await_response(socket, buffer).await?;
                Ok(msg)
            } else {
                Err(Error::AccessSocket)
            }
        })
    }

    #[tracing::instrument]
    #[inline]
    pub fn topics(&self) -> Result<Msg<Vec<String>>, Error> {
        let packet: GenericMsg = GenericMsg {
            msg_type: MsgType::TOPICS,
            timestamp: Utc::now(),
            topic: "".to_string(),
            data_type: std::any::type_name::<()>().to_string(),
            data: Vec::new(),
        };

        let packet_as_bytes: Vec<u8> = to_allocvec(&packet)?;
        let buffer = self.buffer.clone();

        self.rt_handle.block_on(async {
            if let Some(socket) = &self.socket {
                send_msg(socket, packet_as_bytes, self.cfg.network_cfg.host_addr).await?;
                let msg = await_response(socket, buffer).await?;
                Ok(msg)
            } else {
                Err(Error::AccessSocket)
            }
        })
    }
}
