use crate::node::network_config::Nonblocking;
use crate::node::tcp::*;
use crate::node::{Active, Node};
use crate::prelude::*;
use crate::*;

use std::convert::TryInto;
use std::ops::DerefMut;

use chrono::Utc;

use postcard::{from_bytes, to_allocvec};
#[cfg(feature = "quic")]
use quinn::Connection as QuicConnection;
use std::result::Result;
use tracing::*;

use crate::node::network_config::{Interface, Tcp};

impl<T: Message + 'static> Node<Nonblocking, Tcp, Active, T> {
    // TO_DO: The error handling in the async blocks need to be improved
    /// Send data to host on Node's assigned topic using `Msg<T>` packet
    #[tracing::instrument]
    #[inline]
    pub async fn publish(&self, val: T) -> Result<(), Error> {
        let msg: Msg<T> = Msg::new(MsgType::Set, self.topic.clone(), val);
        let packet = msg.to_generic()?.as_bytes()?;

        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        // Send the publish message
        send_msg(stream, packet).await?;
        Ok(())
    }

    #[tracing::instrument]
    #[inline]
    pub async fn publish_msg(&self, msg: Msg<T>) -> Result<(), Error> {
        let packet = msg.to_generic()?.as_bytes()?;

        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        // Send the publish message
        send_msg(stream, packet).await?;
        Ok(())
    }

    /// Request data from host on Node's assigned topic
    #[tracing::instrument]
    #[inline]
    pub async fn request(&self) -> Result<Msg<T>, Error> {
        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        let packet = GenericMsg::get::<T>(self.topic.clone());
        let packet_as_bytes: Vec<u8> = to_allocvec(&packet)?;

        let mut buffer = self.buffer.lock().await;
        send_msg(stream, packet_as_bytes).await?;
        let msg = await_response::<T>(stream, &mut buffer).await?;
        Ok(msg)
    }

    #[tracing::instrument]
    #[inline]
    pub async fn topics(&self) -> Result<Msg<Vec<String>>, Error> {
        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        let packet = GenericMsg::topics();
        let packet_as_bytes: Vec<u8> = to_allocvec(&packet)?;

        let mut buffer = self.buffer.lock().await;
        send_msg(stream, packet_as_bytes).await?;
        let msg = await_response::<Vec<String>>(stream, &mut buffer).await?;
        Ok(msg)
    }
}

use crate::node::network_config::Blocking;

impl<T: Message + 'static> Node<Blocking, Tcp, Active, T> {
    // TO_DO: The error handling in the async blocks need to be improved
    /// Send data to host on Node's assigned topic using `Msg<T>` packet
    #[tracing::instrument]
    #[inline]
    pub fn publish(&self, val: T) -> Result<(), Error> {
        let msg = Msg::new(MsgType::Set, self.topic.clone(), val);
        let generic: GenericMsg = msg.try_into()?;

        let packet_as_bytes: Vec<u8> = to_allocvec(&generic)?;

        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };

        handle.block_on(async {
            // Send the publish message
            send_msg(stream, packet_as_bytes).await?;
            Ok(())
        })
    }

    #[tracing::instrument]
    #[inline]
    pub fn publish_msg(&self, msg: Msg<T>) -> Result<(), Error> {
        let generic: GenericMsg = msg.try_into()?;
        let packet_as_bytes: Vec<u8> = to_allocvec(&generic)?;

        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };

        handle.block_on(async {
            // Send the publish message
            send_msg(stream, packet_as_bytes).await?;
            Ok(())
        })
    }

    /// Request data from host on Node's assigned topic
    #[tracing::instrument]
    #[inline]
    pub fn request(&self) -> Result<Msg<T>, Error> {
        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        let packet = GenericMsg::get::<T>(&self.topic);

        let packet_as_bytes: Vec<u8> = to_allocvec(&packet)?;

        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };

        handle.block_on(async {
            let mut buffer = self.buffer.lock().await;
            send_msg(stream, packet_as_bytes).await?;
            let msg = await_response::<T>(stream, &mut buffer).await?;
            Ok(msg)
        })
    }

    #[tracing::instrument]
    #[inline]
    pub fn topics(&self) -> Result<Msg<Vec<String>>, Error> {
        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        let packet = GenericMsg::topics().as_bytes()?;

        if let Some(handle) = &self.rt_handle {
            handle.block_on(async {
                let mut buffer = self.buffer.lock().await;
                send_msg(stream, packet).await?;
                let msg = await_response::<Vec<String>>(stream, &mut buffer).await?;
                Ok(msg)
            })
        } else {
            Err(Error::HandleAccess)
        }
    }
}
