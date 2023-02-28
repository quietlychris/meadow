use crate::node::Interface;
use crate::Error;
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
        let val_vec: Vec<u8> = match to_allocvec(&val) {
            Ok(val_vec) => val_vec,
            Err(_e) => return Err(Error::Serialization),
        };

        let packet = GenericMsg {
            msg_type: MsgType::SET,
            timestamp: Utc::now(),
            name: self.name.to_string(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: val_vec.to_vec(),
        };

        let packet_as_bytes: Vec<u8> = match to_allocvec(&packet) {
            Ok(packet) => packet,
            Err(_e) => return Err(Error::Serialization),
        };

        if let Some(connection) = &self.connection {
            self.runtime.block_on(async {
                match connection.open_bi().await {
                    Ok((mut send, _recv)) => {
                        info!("Node succesfully opened stream from connection");
                        send.write_all(&packet_as_bytes).await.unwrap();
                        send.finish().await.unwrap();
                    }
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                };

                Ok(())
            })
        } else {
            Err(Error::QuicIssue)
        }
    }

    pub fn request(&self) -> Result<T, Error> {
        let packet = GenericMsg {
            msg_type: MsgType::GET,
            timestamp: Utc::now(),
            name: self.name.to_string(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: Vec::new(),
        };

        let packet_as_bytes: Vec<u8> = match to_allocvec(&packet) {
            Ok(packet) => packet,
            Err(_e) => return Err(Error::Serialization),
        };

        self.runtime.block_on(async {
            let mut buf = vec![0u8; self.cfg.network_cfg.max_buffer_size];

            if let Some(connection) = self.connection.clone() {
                let reply = match connection.open_bi().await {
                    Ok((mut send, mut recv)) => {
                        info!("Node succesfully opened stream from connection");
                        send.write_all(&packet_as_bytes).await.unwrap();
                        send.finish().await.unwrap();

                        match recv.read(&mut buf).await {
                            //Ok(0) => Err(Error::QuicIssue),
                            Ok(Some(n)) => {
                                let bytes = &buf[..n];

                                // let msg: Result<GenericMsg, postcard::Error> = from_bytes::<T>(bytes);
                                match from_bytes::<GenericMsg>(bytes) {
                                    Ok(reply) => Ok(reply),
                                    Err(_) => Err(Error::Deserialization),
                                }
                            }
                            _ => {
                                // if e.kind() == std::io::ErrorKind::WouldBlock {}
                                Err(Error::QuicIssue)
                            }
                        }
                    }
                    _ => Err(Error::QuicIssue),
                };

                if let Ok(msg) = reply {
                    match from_bytes::<T>(&msg.data) {
                        Ok(data) => Ok(data),
                        Err(_e) => Err(Error::Deserialization),
                    }
                } else {
                    Err(Error::QuicIssue)
                }
            } else {
                Err(Error::QuicIssue)
            }
        })
    }
}
