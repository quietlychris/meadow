use crate::node::network_config::{Nonblocking, Udp};
use crate::node::Interface;
use crate::node::Node;
use crate::node::{Active, Idle};
use crate::prelude::*;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

use crate::node::udp::*;

use chrono::Utc;

use postcard::{from_bytes, to_allocvec};
#[cfg(feature = "quic")]
use quinn::Connection as QuicConnection;
use std::result::Result;
use tracing::*;

impl<T: Message> From<Node<Nonblocking, Udp, Idle, T>> for Node<Nonblocking, Udp, Active, T> {
    fn from(node: Node<Nonblocking, Udp, Idle, T>) -> Self {
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

use crate::node::Block;
use std::fmt::Debug;

impl<T: Message + 'static, B: Block + Debug> Node<B, Udp, Active, T> {
    #[tracing::instrument]
    #[inline]
    async fn publish_internal(&self, val: T) -> Result<(), Error> {
        let packet = Msg::new(MsgType::Set, self.topic.clone(), val)
            .to_generic()?
            .as_bytes()?;

        let socket = match self.socket.as_ref() {
            Some(socket) => socket,
            None => return Err(Error::AccessSocket),
        };

        socket
            .send_to(&packet, self.cfg.network_cfg.host_addr)
            .await?;
        Ok(())
    }

    #[tracing::instrument]
    #[inline]
    async fn publish_msg_internal(&self, msg: Msg<T>) -> Result<(), Error> {
        let packet = msg.to_generic()?.as_bytes()?;
        let socket = match self.socket.as_ref() {
            Some(socket) => socket,
            None => return Err(Error::AccessSocket),
        };

        socket
            .send_to(&packet, self.cfg.network_cfg.host_addr)
            .await?;
        Ok(())
    }

    #[tracing::instrument]
    #[inline]
    async fn request_nth_back_internal(&self, n: usize) -> Result<Msg<T>, Error> {
        let packet = GenericMsg::get_nth::<T>(self.topic.clone(), n).as_bytes()?;
        let buffer = self.buffer.clone();

        if let Some(socket) = &self.socket {
            send_msg(socket, packet, self.cfg.network_cfg.host_addr).await?;
            let msg = await_response(socket, buffer).await?.try_into()?;
            Ok(msg)
        } else {
            Err(Error::AccessSocket)
        }
    }

    #[tracing::instrument]
    #[inline]
    async fn topics_internal(&self) -> Result<Msg<Vec<String>>, Error> {
        let packet = GenericMsg::topics().as_bytes()?;
        let buffer = self.buffer.clone();

        if let Some(socket) = &self.socket {
            send_msg(socket, packet, self.cfg.network_cfg.host_addr).await?;
            let msg = await_response(socket, buffer).await?.try_into()?;
            Ok(msg)
        } else {
            Err(Error::AccessSocket)
        }
    }
}

impl<T: Message + 'static> Node<Nonblocking, Udp, Active, T> {
    #[tracing::instrument]
    #[inline]
    pub async fn publish(&self, val: T) -> Result<(), Error> {
        self.publish_internal(val).await?;
        Ok(())
    }

    pub async fn publish_msg(&self, msg: Msg<T>) -> Result<(), Error> {
        self.publish_msg_internal(msg).await?;
        Ok(())
    }

    #[tracing::instrument]
    #[inline]
    pub async fn request(&self) -> Result<Msg<T>, Error> {
        let msg = self.request_nth_back_internal(0).await?;
        Ok(msg)
    }

    #[tracing::instrument]
    #[inline]
    pub async fn topics(&self) -> Result<Msg<Vec<String>>, Error> {
        let msg = self.topics_internal().await?;
        Ok(msg)
    }
}

//--------

use crate::node::network_config::Blocking;

impl<T: Message> From<Node<Blocking, Udp, Idle, T>> for Node<Blocking, Udp, Active, T> {
    fn from(node: Node<Blocking, Udp, Idle, T>) -> Self {
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

impl<T: Message + 'static> Node<Blocking, Udp, Active, T> {
    #[tracing::instrument]
    #[inline]
    pub fn publish(&self, val: T) -> Result<(), Error> {
        match &self.rt_handle {
            Some(handle) => handle.block_on(async {
                self.publish_internal(val).await?;
                Ok(())
            }),
            None => Err(Error::HandleAccess),
        }
    }

    #[tracing::instrument]
    #[inline]
    pub fn publish_msg(&self, msg: Msg<T>) -> Result<(), Error> {
        match &self.rt_handle {
            Some(handle) => handle.block_on(async {
                self.publish_msg_internal(msg).await?;
                Ok(())
            }),
            None => Err(Error::HandleAccess),
        }
    }

    #[tracing::instrument]
    #[inline]
    pub fn request(&self) -> Result<Msg<T>, Error> {
        match &self.rt_handle {
            Some(handle) => handle.block_on(async {
                let msg = self.request_nth_back_internal(0).await?;
                Ok(msg)
            }),
            None => Err(Error::HandleAccess),
        }
    }

    #[tracing::instrument]
    #[inline]
    pub fn request_nth_back(&self, n: usize) -> Result<Msg<T>, Error> {
        match &self.rt_handle {
            Some(handle) => handle.block_on(async {
                let msg = self.request_nth_back_internal(n).await?;
                Ok(msg)
            }),
            None => Err(Error::HandleAccess),
        }
    }

    #[tracing::instrument]
    #[inline]
    pub fn topics(&self) -> Result<Msg<Vec<String>>, Error> {
        match &self.rt_handle {
            Some(handle) => handle.block_on(async {
                let msg = self.topics_internal().await?;
                Ok(msg)
            }),
            None => Err(Error::HandleAccess),
        }
    }
}
