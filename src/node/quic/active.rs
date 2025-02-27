use crate::error::{Error, Quic::*};
use crate::node::network_config::{Nonblocking, Quic};
use crate::node::Active;
use crate::node::Node;
use crate::prelude::*;

use crate::msg::{GenericMsg, Message, Msg};
use std::convert::TryInto;

use chrono::Utc;

use crate::node::Block;
use postcard::*;
use quinn::Connection as QuicConnection;
use std::fmt::Debug;
use std::result::Result;
use tracing::*;

impl<T: Message + 'static, B: Block + Debug> Node<B, Quic, Active, T> {
    #[tracing::instrument(skip_all)]
    #[inline]
    async fn publish_internal(&self, val: T) -> Result<(), Error> {
        let packet = Msg::new(MsgType::Set, self.topic.clone(), val)
            .to_generic()?
            .as_bytes()?;

        if let Some(connection) = &self.connection {
            match connection.open_bi().await {
                Ok((mut send, _recv)) => {
                    debug!("Node succesfully opened stream from connection");

                    if let Ok(()) = send.write_all(&packet).await {
                        if let Ok(()) = send.finish().await {
                            debug!("Node successfully wrote packet to stream");
                        }
                    } else {
                        error!("Error writing packet to stream");
                    }
                }
                Err(e) => {
                    warn!("{:?}", e);
                }
            };

            Ok(())
        } else {
            Err(Error::Quic(Connection))
        }
    }

    #[tracing::instrument(skip_all)]
    #[inline]
    async fn publish_msg_internal(&self, msg: Msg<T>) -> Result<(), Error> {
        let packet = msg.to_generic()?.as_bytes()?;

        if let Some(connection) = &self.connection {
            match connection.open_bi().await {
                Ok((mut send, _recv)) => {
                    debug!("Node succesfully opened stream from connection");

                    if let Ok(()) = send.write_all(&packet).await {
                        if let Ok(()) = send.finish().await {
                            debug!("Node successfully wrote packet to stream");
                        }
                    } else {
                        error!("Error writing packet to stream");
                    }
                }
                Err(e) => {
                    warn!("{:?}", e);
                }
            };

            Ok(())
        } else {
            Err(Error::Quic(Connection))
        }
    }

    #[tracing::instrument(skip_all)]
    #[inline]
    async fn request_nth_back_internal(&self, n: usize) -> Result<Msg<T>, Error> {
        let packet = GenericMsg::get_nth::<T>(self.topic.clone(), n).as_bytes()?;

        let mut buf = self.buffer.lock().await;

        if let Some(connection) = self.connection.clone() {
            let (mut send, mut recv) = connection.open_bi().await?;
            debug!("Node succesfully opened stream from connection");
            send.write_all(&packet).await?;
            // send.finish().await.map_err(WriteError)?;

            loop {
                match recv.read(&mut buf).await? {
                    Some(0) => continue,
                    Some(n) => {
                        let bytes = &buf[..n];
                        let generic = from_bytes::<GenericMsg>(bytes)?;
                        let msg = generic.try_into()?;

                        return Ok(msg);
                    }
                    None => continue,
                }
            }
        } else {
            Err(Error::Quic(Connection))
        }
    }

    #[tracing::instrument(skip_all)]
    #[inline]
    async fn topics_internal(&self) -> Result<Msg<Vec<String>>, Error> {
        let packet = GenericMsg::topics().as_bytes()?;

        let mut buf = self.buffer.lock().await;

        let connection = self.connection.clone().ok_or(Connection)?;

        let (mut send, mut recv) = connection.open_bi().await?;
        debug!("Node succesfully opened stream from connection");
        send.write_all(&packet).await?;
        send.finish().await?;

        let n = recv.read(&mut buf).await?.ok_or(Connection)?;
        let bytes = &buf[..n];
        let reply = from_bytes::<GenericMsg>(bytes)?;
        let topics: Msg<Vec<String>> = reply.try_into()?;
        Ok(topics)
    }
}

impl<T: Message + 'static> Node<Nonblocking, Quic, Active, T> {
    #[tracing::instrument(skip_all)]
    #[inline]
    pub async fn publish(&self, val: T) -> Result<(), Error> {
        self.publish_internal(val).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    #[inline]
    pub async fn publish_msg(&self, msg: Msg<T>) -> Result<(), Error> {
        self.publish_msg_internal(msg).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    #[inline]
    pub async fn request(&self) -> Result<Msg<T>, Error> {
        let msg = self.request_nth_back_internal(0).await?;
        Ok(msg)
    }

    #[tracing::instrument(skip_all)]
    #[inline]
    pub async fn topics(&self) -> Result<Msg<Vec<String>>, Error> {
        let msg = self.topics_internal().await?;
        Ok(msg)
    }
}

//-----

use crate::node::network_config::Blocking;

impl<T: Message + 'static> Node<Blocking, Quic, Active, T> {
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

    #[tracing::instrument(skip_all)]
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

    #[tracing::instrument(skip_all)]
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

    #[tracing::instrument(skip_all)]
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

    #[tracing::instrument(skip_all)]
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
