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

impl<T: Message + 'static> Node<Nonblocking, Udp, Active, T> {
    #[tracing::instrument]
    #[inline]
    pub async fn publish(&self, val: T) -> Result<(), Error> {
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

    pub async fn publish_msg(&self, msg: Msg<T>) -> Result<(), Error> {
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
    pub async fn request(&self) -> Result<Msg<T>, Error> {
        let packet = GenericMsg::get::<T>(self.topic.clone()).as_bytes()?;
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
    pub async fn topics(&self) -> Result<Msg<Vec<String>>, Error> {
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
        let packet = Msg::new(MsgType::Set, self.topic.clone(), val)
            .to_generic()?
            .as_bytes()?;

        let socket = match self.socket.as_ref() {
            Some(socket) => socket,
            None => return Err(Error::AccessSocket),
        };

        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };

        handle.block_on(async {
            socket
                .send_to(&packet, self.cfg.network_cfg.host_addr)
                .await?;
            Ok(())
        })
    }

    #[tracing::instrument]
    #[inline]
    pub fn publish_msg(&self, msg: Msg<T>) -> Result<(), Error> {
        let packet = msg.to_generic()?.as_bytes()?;

        let socket = match self.socket.as_ref() {
            Some(socket) => socket,
            None => return Err(Error::AccessSocket),
        };

        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };

        handle.block_on(async {
            socket
                .send_to(&packet, self.cfg.network_cfg.host_addr)
                .await?;
            Ok(())
        })
    }

    #[tracing::instrument]
    #[inline]
    pub fn request(&self) -> Result<Msg<T>, Error> {
        let packet = GenericMsg::get::<T>(self.topic.clone()).as_bytes()?;
        let buffer = self.buffer.clone();

        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };

        handle.block_on(async {
            if let Some(socket) = &self.socket {
                send_msg(socket, packet, self.cfg.network_cfg.host_addr).await?;
                let msg = await_response(socket, buffer).await?.try_into()?;
                Ok(msg)
            } else {
                Err(Error::AccessSocket)
            }
        })
    }

    #[tracing::instrument]
    #[inline]
    pub fn request_nth_back(&self, n: usize) -> Result<Msg<T>, Error> {
        let packet = GenericMsg::get_nth::<T>(self.topic.clone(), n).as_bytes()?;
        let buffer = self.buffer.clone();

        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };

        handle.block_on(async {
            if let Some(socket) = &self.socket {
                send_msg(socket, packet, self.cfg.network_cfg.host_addr).await?;
                let msg = await_response(socket, buffer).await?.try_into()?;
                Ok(msg)
            } else {
                Err(Error::AccessSocket)
            }
        })
    }

    #[tracing::instrument]
    #[inline]
    pub fn topics(&self) -> Result<Msg<Vec<String>>, Error> {
        let packet = GenericMsg::topics().as_bytes()?;
        let buffer = self.buffer.clone();

        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };

        handle.block_on(async {
            if let Some(socket) = &self.socket {
                send_msg(socket, packet, self.cfg.network_cfg.host_addr).await?;
                let msg = await_response(socket, buffer).await?.try_into()?;
                Ok(msg)
            } else {
                Err(Error::AccessSocket)
            }
        })
    }
}
