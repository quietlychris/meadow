use crate::*;
use crate::{error::HostOperation, Error};

use std::convert::TryInto;
use std::ops::DerefMut;

use chrono::Utc;

use postcard::*;
#[cfg(feature = "quic")]
use quinn::Connection as QuicConnection;
use std::result::Result;
use tracing::*;

use crate::node::blocking::network_config::{Interface, Tcp};

/// Tcp implements the Interface trait
impl Interface for Tcp {}

impl<T: Message + 'static> Node<Tcp, Active, T> {
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

        self.rt_handle.block_on(async {
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

        self.rt_handle.block_on(async {
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

        self.rt_handle.block_on(async {
            let mut buffer = self.buffer.lock().await;
            send_msg(stream, packet_as_bytes).await?;
            let msg = await_response::<Vec<String>>(stream, &mut buffer).await?;
            Ok(msg)
        })
    }
}
