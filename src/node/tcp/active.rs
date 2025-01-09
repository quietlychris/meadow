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

impl<T: Message + 'static> Node<Nonblocking, Tcp, Active, T> {
    // TO_DO: The error handling in the async blocks need to be improved
    /// Send data to host on Node's assigned topic using `Msg<T>` packet
    #[tracing::instrument]
    #[inline]
    pub async fn publish(&self, val: T) -> Result<(), Error> {
        let data: Vec<u8> = to_allocvec(&val)?;

        let generic = GenericMsg {
            msg_type: MsgType::SET,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data,
        };

        let packet_as_bytes: Vec<u8> = to_allocvec(&generic)?;

        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        // Send the publish message
        send_msg(stream, packet_as_bytes).await?;

        // Wait for the publish acknowledgement
        let mut buf = self.buffer.lock().await;
        loop {
            if let Ok(()) = stream.readable().await {
                match stream.try_read(&mut buf) {
                    Ok(0) => continue,
                    Ok(n) => {
                        let bytes = &buf[..n];
                        if let Ok(HostOperation::FAILURE) = from_bytes::<HostOperation>(bytes) {
                            error!("Host-side error on publish");
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
    pub async fn publish_raw_msg(&self, msg: Msg<T>) -> Result<(), Error> {
        let generic: GenericMsg = msg.try_into()?;
        let packet_as_bytes: Vec<u8> = to_allocvec(&generic)?;

        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        // Send the publish message
        send_msg(stream, packet_as_bytes).await?;

        // Wait for the publish acknowledgement
        let mut buf = self.buffer.lock().await;
        loop {
            if let Ok(()) = stream.readable().await {
                match stream.try_read(&mut buf) {
                    Ok(0) => continue,
                    Ok(n) => {
                        let bytes = &buf[..n];
                        if let Ok(HostOperation::FAILURE) = from_bytes::<HostOperation>(bytes) {
                            error!("Host-side error on publish");
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
    pub async fn request(&self) -> Result<Msg<T>, Error> {
        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        let packet: GenericMsg = GenericMsg {
            msg_type: MsgType::GET,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: Vec::new(),
        };

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

        let packet: GenericMsg = GenericMsg {
            msg_type: MsgType::TOPICS,
            timestamp: Utc::now(),
            topic: "".to_string(),
            data_type: std::any::type_name::<()>().to_string(),
            data: Vec::new(),
        };

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
        let data: Vec<u8> = to_allocvec(&val)?;

        let generic = GenericMsg {
            msg_type: MsgType::SET,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data,
        };

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

            // Wait for the publish acknowledgement
            let mut buf = self.buffer.lock().await;
            loop {
                if let Ok(()) = stream.readable().await {
                    match stream.try_read(&mut buf) {
                        Ok(0) => continue,
                        Ok(n) => {
                            let bytes = &buf[..n];
                            if let Ok(HostOperation::FAILURE) = from_bytes::<HostOperation>(bytes) {
                                error!("Host-side error on publish");
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
        })
    }

    #[tracing::instrument]
    #[inline]
    pub fn publish_raw_msg(&self, msg: Msg<T>) -> Result<(), Error> {
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

            // Wait for the publish acknowledgement
            let mut buf = self.buffer.lock().await;
            loop {
                if let Ok(()) = stream.readable().await {
                    match stream.try_read(&mut buf) {
                        Ok(0) => continue,
                        Ok(n) => {
                            let bytes = &buf[..n];
                            if let Ok(HostOperation::FAILURE) = from_bytes::<HostOperation>(bytes) {
                                error!("Host-side error on publish");
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

        let packet: GenericMsg = GenericMsg {
            msg_type: MsgType::GET,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: Vec::new(),
        };

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

        let packet: GenericMsg = GenericMsg {
            msg_type: MsgType::TOPICS,
            timestamp: Utc::now(),
            topic: "".to_string(),
            data_type: std::any::type_name::<()>().to_string(),
            data: Vec::new(),
        };

        let packet_as_bytes: Vec<u8> = to_allocvec(&packet)?;

        if let Some(handle) = &self.rt_handle {
            handle.block_on(async {
                let mut buffer = self.buffer.lock().await;
                send_msg(stream, packet_as_bytes).await?;
                let msg = await_response::<Vec<String>>(stream, &mut buffer).await?;
                Ok(msg)
            })
        } else {
            Err(Error::HandleAccess)
        }
    }
}
