use crate::node::network_config::Udp;
use crate::node::Interface;
use crate::node::Node;
use crate::Error;
use crate::{Active, Idle, MsgType};
use std::marker::PhantomData;
use std::ops::DerefMut;

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
    pub fn request(&self) -> Result<Msg<T>, Error> {
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
            if let Some(socket) = &self.socket {
                if let Ok(_n) =
                    send_msg(socket, packet_as_bytes, self.cfg.network_cfg.host_addr).await
                {
                    let mut buffer = self.buffer.lock().await;
                    match await_response(socket, &mut buffer).await {
                        Ok(msg) => Ok(msg.clone()),
                        Err(_e) => Err(Error::Deserialization),
                    }
                } else {
                    Err(Error::BadResponse)
                }
            } else {
                Err(Error::AccessSocket)
            }
        })
    }
}

use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

#[inline]
pub async fn await_response<T: Message>(
    socket: &UdpSocket,
    buf: &mut [u8], //max_buffer_size: usize,
) -> Result<Msg<T>, Error> {
    // Read the requested data into a buffer
    match socket.readable().await {
        Ok(_) => (),
        Err(_e) => return Err(Error::AccessSocket),
    };

    loop {
        match socket.try_recv(buf) {
            Ok(0) => continue,
            Ok(n) => {
                let bytes = &buf[..n];
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
                // if e.kind() == std::io::ErrorKind::WouldBlock {println!("Would block");}
                continue;
            }
        }
    }
}

use std::net::SocketAddr;

#[inline]
async fn send_msg(
    socket: &UdpSocket,
    packet_as_bytes: Vec<u8>,
    host_addr: SocketAddr,
) -> Result<usize, Error> {
    match socket.writable().await {
        Ok(_) => (),
        Err(_e) => return Err(Error::AccessSocket),
    };

    // Write the request
    for _ in 0..10 {
        match socket.send_to(&packet_as_bytes, host_addr).await {
            Ok(n) => return Ok(n),
            Err(_e) => {}
        }
    }
    Err(Error::BadResponse)
}
