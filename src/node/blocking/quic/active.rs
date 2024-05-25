use crate::error::{Error, Quic::*};
use crate::node::blocking::network_config::Quic;
use crate::node::blocking::Interface;
use crate::*;

use crate::msg::{GenericMsg, Message, Msg};
use std::convert::TryInto;

use chrono::Utc;

use postcard::*;
use quinn::Connection as QuicConnection;
use std::result::Result;
use tracing::*;

/// Quic implements the Interface trait
impl Interface for Quic {}

impl<T: Message + 'static> Node<Quic, Active, T> {
    #[tracing::instrument(skip(self))]
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

        if let Some(connection) = &self.connection {
            self.rt_handle.block_on(async {
                match connection.open_bi().await {
                    Ok((mut send, _recv)) => {
                        debug!("Node succesfully opened stream from connection");

                        if let Ok(()) = send.write_all(&packet_as_bytes).await {
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
            })
        } else {
            Err(Error::Quic(Connection))
        }
    }

    pub fn request(&self) -> Result<Msg<T>, Error> {
        let packet = GenericMsg {
            msg_type: MsgType::GET,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: Vec::new(),
        };

        let packet_as_bytes: Vec<u8> = to_allocvec(&packet)?;

        self.rt_handle.block_on(async {
            let mut buf = self.buffer.lock().await;

            if let Some(connection) = self.connection.clone() {
                let (mut send, mut recv) = connection.open_bi().await.map_err(ConnectionError)?;
                debug!("Node succesfully opened stream from connection");
                send.write_all(&packet_as_bytes).await.map_err(WriteError)?;
                // send.finish().await.map_err(WriteError)?;

                loop {
                    match recv.read(&mut buf).await.map_err(ReadError)? {
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
        })
    }

    pub fn topics(&self) -> Result<Msg<Vec<String>>, Error> {
        let packet = GenericMsg {
            msg_type: MsgType::TOPICS,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<()>().to_string(),
            data: Vec::new(),
        };

        let packet_as_bytes: Vec<u8> = to_allocvec(&packet)?;

        self.rt_handle.block_on(async {
            let mut buf = self.buffer.lock().await;

            let connection = self.connection.clone().ok_or(Connection)?;

            let (mut send, mut recv) = connection.open_bi().await.map_err(ConnectionError)?;
            debug!("Node succesfully opened stream from connection");
            send.write_all(&packet_as_bytes).await.map_err(WriteError)?;
            send.finish().await.map_err(WriteError)?;

            let n = recv
                .read(&mut buf)
                .await
                .map_err(ReadError)?
                .ok_or(Connection)?;
            let bytes = &buf[..n];
            let reply = from_bytes::<GenericMsg>(bytes)?;
            let topics: Msg<Vec<String>> = reply.try_into()?;
            Ok(topics)
        })
    }
}
