use crate::error::{Error, Quic::*};
use crate::node::network_config::Quic;
use crate::node::Interface;
use crate::*;

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
            self.runtime.block_on(async {
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

    pub fn request(&self) -> Result<T, Error> {
        let packet = GenericMsg {
            msg_type: MsgType::GET,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: Vec::new(),
        };

        let packet_as_bytes: Vec<u8> = to_allocvec(&packet)?;

        self.runtime.block_on(async {
            let mut buf = self.buffer.lock().await;

            if let Some(connection) = self.connection.clone() {
                let reply = match connection.open_bi().await {
                    Ok((mut send, mut recv)) => {
                        debug!("Node succesfully opened stream from connection");
                        if let Ok(()) = send.write_all(&packet_as_bytes).await {
                            if let Ok(()) = send.finish().await {
                                debug!("Node successfully wrote packet to stream");
                            }
                        } else {
                            error!("Error writing packet to stream");
                        }

                        match recv.read(&mut buf).await {
                            //Ok(0) => Err(Error::QuicIssue),
                            Ok(Some(n)) => {
                                let bytes = &buf[..n];
                                let reply = from_bytes::<GenericMsg>(bytes)?;
                                Ok(reply)
                            }
                            _ => {
                                // // if e.kind() == std::io::ErrorKind::WouldBlock {}
                                Err(Error::Quic(RecvRead))
                            }
                        }
                    }
                    _ => Err(Error::Quic(OpenBi)),
                };

                if let Ok(msg) = reply {
                    let data = from_bytes::<T>(&msg.data)?;
                    Ok(data)
                } else {
                    Err(Error::Quic(BadGenericMsg))
                }
            } else {
                Err(Error::Quic(Connection))
            }
        })
    }

    pub fn topics(&self) -> Result<Vec<String>, Error> {
        let packet = GenericMsg {
            msg_type: MsgType::TOPICS,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<()>().to_string(),
            data: Vec::new(),
        };

        let packet_as_bytes: Vec<u8> = to_allocvec(&packet)?;

        self.runtime.block_on(async {
            let mut buf = self.buffer.lock().await;

            if let Some(connection) = self.connection.clone() {
                let (mut send, mut recv) = connection.open_bi().await.map_err(ConnectionError)?;
                debug!("Node succesfully opened stream from connection");
                send.write_all(&packet_as_bytes).await.map_err(WriteError)?;
                send.finish().await.map_err(WriteError)?;

                if let Some(n) = recv.read(&mut buf).await.map_err(ReadError)? {
                    let bytes = &buf[..n];
                    let reply = from_bytes::<GenericMsg>(bytes)?;
                    let topics = from_bytes::<Vec<String>>(&reply.data)?;
                    Ok(topics)
                } else {
                    Ok(Vec::new())
                }
            } else {
                Ok(Vec::new())
            }
        })
    }
}
