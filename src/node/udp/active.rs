use crate::node::network_config::Udp;
use crate::node::Interface;
use crate::node::Node;
use crate::Error;
use crate::{Active, Idle, MsgType};
use std::marker::PhantomData;

use crate::node::udp::*;

use chrono::Utc;

use postcard::*;
#[cfg(feature = "quic")]
use quinn::Connection as QuicConnection;
use std::result::Result;
use tracing::*;

/// Udp implements the Interface trait
impl Interface for Udp {}

impl<T: Message> From<Node<Udp, Idle, T>> for Node<Udp, Active, T> {
    fn from(node: Node<Udp, Idle, T>) -> Self {
        Self {
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
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

impl<T: Message + 'static> Node<Udp, Active, T> {
    #[tracing::instrument]
    #[inline]
    pub fn publish(&self, val: T) -> Result<(), Error> {
        let data: Vec<u8> = match to_allocvec(&val) {
            Ok(data) => data,
            Err(_e) => return Err(Error::Serialization),
        };

        let generic = GenericMsg {
            msg_type: MsgType::SET,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data,
        };

        let packet_as_bytes: Vec<u8> = match to_allocvec(&generic) {
            Ok(packet) => packet,
            Err(_e) => return Err(Error::Serialization),
        };

        let socket = match self.socket.as_ref() {
            Some(socket) => socket,
            None => return Err(Error::AccessSocket),
        };

        self.runtime.block_on(async {
            match socket
                .send_to(&packet_as_bytes, self.cfg.network_cfg.host_addr)
                .await
            {
                Ok(_len) => Ok(()),
                Err(e) => {
                    error!("{:?}", e);
                    Err(Error::UdpSend)
                }
            }
        })
    }

    #[tracing::instrument]
    #[inline]
    pub fn request(&mut self) -> Result<Msg<T>, Error> {
        let socket = match self.socket.as_ref() {
            Some(socket) => socket,
            None => return Err(Error::AccessSocket),
        };

        let packet: GenericMsg = GenericMsg {
            msg_type: MsgType::GET,
            timestamp: Utc::now(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: Vec::new(),
        };

        let packet_as_bytes: Vec<u8> = match to_allocvec(&packet) {
            Ok(packet) => packet,
            Err(_e) => return Err(Error::Serialization),
        };

        self.runtime.block_on(async {
            if let Ok(_n) = &self.send_msg(packet_as_bytes).await {
                match &self.await_response(&self.buffer).await {
                    Ok(msg) => Ok(msg.clone()),
                    Err(_e) => Err(Error::Deserialization),
                }
            } else {
                Err(Error::BadResponse)
            }
        })
    }

    #[inline]
    async fn send_msg(&self, packet_as_bytes: Vec<u8>) -> Result<usize, Error> {
        match &self.socket {
            Some(socket) => {
                match socket.writable().await {
                    Ok(_) => (),
                    Err(_e) => return Err(Error::AccessSocket),
                };

                // Write the request
                for _ in 0..10 {
                    match socket
                        .send_to(&packet_as_bytes, self.cfg.network_cfg.host_addr)
                        .await
                    {
                        Ok(n) => return Ok(n),
                        Err(_e) => {}
                    }
                }
                Err(Error::BadResponse)
            }
            None => Err(Error::AccessSocket),
        }
    }

    #[inline]
    async fn await_response(
        &self,
        buffer: &mut Vec<u8>,
        // max_buffer_size: usize,
    ) -> Result<Msg<T>, Error> {
        // Read the requested data into a buffer
        // TO_DO: Having to re-allocate this each time isn't very efficient
        // let mut buf = vec![0u8; max_buffer_size];
        // TO_DO: This can be made cleaner

        match &self.socket {
            Some(socket) => {
                loop {
                    socket.readable().await.unwrap();
                    match socket.try_recv(buffer) {
                        Ok(0) => continue,
                        Ok(n) => {
                            let bytes = &buffer[..n];
                            match postcard::from_bytes::<GenericMsg>(bytes) {
                                Ok(generic) => {
                                    if let Ok(msg) = TryInto::<Msg<T>>::try_into(generic) {
                                        return Ok(msg);
                                    } else {
                                        return Err(Error::Deserialization);
                                    }
                                }
                                Err(_e) => return Err(Error::Deserialization),
                            }
                        }
                        Err(_e) => {
                            // if e.kind() == std::io::ErrorKind::WouldBlock {}
                            debug!("Would block");
                            continue;
                        }
                    }
                }
            },
            None => Err(Error::AccessSocket)
        }


    }
}
