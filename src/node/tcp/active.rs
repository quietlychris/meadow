use crate::error::HostOperation;
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
use crate::node::Block;
use std::fmt::Debug;

impl<T: Message + 'static, B: Block + Debug> Node<B, Tcp, Active, T> {
    #[tracing::instrument]
    #[inline]
    async fn publish_internal(&self, val: T) -> Result<(), Error> {
        let packet = Msg::new(MsgType::Set, self.topic.clone(), val)
            .to_generic()?
            .as_bytes()?;

        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        // Send the publish message
        send_msg(stream, packet).await?;

        // Wait for the publish acknowledgement
        let mut buf = self.buffer.lock().await;
        loop {
            if let Ok(()) = stream.readable().await {
                match stream.try_read(&mut buf) {
                    Ok(0) => continue,
                    Ok(n) => {
                        let bytes = &buf[..n];
                        match from_bytes::<GenericMsg>(bytes) {
                            Ok(g) => match g.msg_type {
                                MsgType::Result(result) => {
                                    if let Err(e) = result {
                                        error!("{}", e);
                                    }
                                }
                                _ => {
                                    info!("{:?}", &g);
                                }
                            },
                            Err(e) => {
                                error!("{}", e);
                            }
                        }

                        break;
                    }
                    Err(_e) => {
                        // if e.kind() == std::io::ErrorKind::WouldBlock {}
                        continue;
                    }
                }
            }
        }
        Ok(())
    }

    #[tracing::instrument]
    #[inline]
    async fn publish_msg_internal(&self, msg: Msg<T>) -> Result<(), Error> {
        let packet = msg.to_generic()?.as_bytes()?;
        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        // Send the publish message
        send_msg(stream, packet).await?;

        // Wait for the publish acknowledgement
        let mut buf = self.buffer.lock().await;
        loop {
            if let Ok(()) = stream.readable().await {
                match stream.try_read(&mut buf) {
                    Ok(0) => continue,
                    Ok(n) => {
                        let bytes = &buf[..n];

                        match from_bytes::<GenericMsg>(bytes) {
                            Ok(g) => match g.msg_type {
                                MsgType::Result(result) => {
                                    if let Err(e) = result {
                                        error!("{}", e)
                                    } else {
                                        info!("{:?}", result);
                                    }
                                }
                                _ => (),
                            },
                            Err(e) => {
                                error!("{}", e);
                            }
                        }

                        break;
                    }
                    Err(_e) => {
                        // if e.kind() == std::io::ErrorKind::WouldBlock {}
                        continue;
                    }
                }
            }
        }
        Ok(())
    }

    /// Request data from host on Node's assigned topic
    #[tracing::instrument]
    #[inline]
    async fn request_nth_back_internal(&self, n: usize) -> Result<Msg<T>, Error> {
        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        let packet = GenericMsg::get_nth::<T>(self.topic.clone(), n).as_bytes()?;

        let mut buffer = self.buffer.lock().await;
        send_msg(stream, packet).await?;
        let msg = await_response(stream, &mut buffer).await?.try_into()?;
        Ok(msg)
    }

    #[tracing::instrument]
    #[inline]
    async fn topics_internal(&self) -> Result<Msg<Vec<String>>, Error> {
        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        let packet = GenericMsg::topics().as_bytes()?;

        let mut buffer = self.buffer.lock().await;
        send_msg(stream, packet).await?;
        let msg = await_response(stream, &mut buffer).await?.try_into()?;
        Ok(msg)
    }
}

impl<T: Message + 'static> Node<Nonblocking, Tcp, Active, T> {
    // TO_DO: The error handling in the async blocks need to be improved
    /// Send data to host on Node's assigned topic using `Msg<T>` packet
    #[tracing::instrument]
    #[inline]
    pub async fn publish(&self, val: T) -> Result<(), Error> {
        self.publish_internal(val).await?;
        Ok(())
    }

    #[tracing::instrument]
    #[inline]
    pub async fn publish_msg(&self, msg: Msg<T>) -> Result<(), Error> {
        self.publish_msg_internal(msg).await?;
        Ok(())
    }

    /// Request data from host on Node's assigned topic
    #[tracing::instrument]
    #[inline]
    pub async fn request(&self) -> Result<Msg<T>, Error> {
        let msg = self.request_nth_back_internal(0).await?;
        Ok(msg)
    }

    /// Request n'th data from host on Node's assigned topic
    #[tracing::instrument]
    #[inline]
    pub async fn request_nth_back(&self, n: usize) -> Result<Msg<T>, Error> {
        let msg = self.request_nth_back_internal(n).await?;
        Ok(msg)
    }

    #[tracing::instrument]
    #[inline]
    pub async fn topics(&self) -> Result<Msg<Vec<String>>, Error> {
        let msg = self.topics_internal().await?;
        Ok(msg)
    }
}

use crate::node::network_config::Blocking;

impl<T: Message + 'static> Node<Blocking, Tcp, Active, T> {
    // TO_DO: The error handling in the async blocks need to be improved
    /// Send data to host on Node's assigned topic using `Msg<T>` packet
    #[tracing::instrument(skip_all)]
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

    /// Request data from host on Node's assigned topic
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

    /// Request data from host on Node's assigned topic
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
