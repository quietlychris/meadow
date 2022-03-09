use crate::*;
use chrono::Utc;
use core::marker::PhantomData;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tokio::net::UdpSocket;

use postcard::*;
use std::error::Error;
use std::result::Result;
use tracing::*;

impl<T: Message + 'static> Node<Active, T> {
    // TO_DO: The error handling in the async blocks need to be improved
    /// Send data to host on Node's assigned topic using Msg<T> packet
    #[tracing::instrument]
    pub fn publish(&self, val: T) -> Result<(), Box<dyn Error>> {
        // let val_vec: heapless::Vec<u8, 4096> = to_vec(&val).unwrap();
        let val_vec: Vec<u8> = to_allocvec(&val).unwrap();

        // println!("Number of bytes in data for {:?} is {}",std::any::type_name::<M>(),val_vec.len());
        let packet = GenericMsg {
            msg_type: MsgType::SET,
            timestamp: Utc::now().to_string(),
            name: self.name.to_string(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: val_vec.to_vec(),
        };
        // info!("The Node's packet to send looks like: {:?}",&packet);

        let packet_as_bytes: Vec<u8> = to_allocvec(&packet).unwrap();
        // info!("Node is publishing: {:?}",&packet_as_bytes);

        let topic = &self.topic;
        let stream = &mut self.stream.as_ref().unwrap();

        let _result = self.runtime.block_on(async {
            send_msg(stream, packet_as_bytes).await.unwrap();

            // Wait for the publish acknowledgement
            let mut buf = [0u8; 1024];
            loop {
                stream.readable().await.unwrap();
                match stream.try_read(&mut buf) {
                    Ok(0) => continue,
                    Ok(n) => {
                        let bytes = &buf[..n];
                        let _msg: Result<String, Box<dyn Error>> = match from_bytes(bytes) {
                            Ok(ack) => {
                                return Ok(ack);
                            }
                            Err(e) => {
                                error!("{}: {:?}", topic, &e);
                                return Err(Box::new(e));
                            }
                        };
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {}
                        continue;
                    }
                }
            }
            Ok(())
        });

        Ok(())
    }

    #[tracing::instrument]
    pub fn publish_udp(&self, val: T) -> Result<(), Box<dyn Error>> {
        // let val_vec: heapless::Vec<u8, 4096> = to_vec(&val).unwrap();
        let val_vec: Vec<u8> = to_allocvec(&val).unwrap();

        // println!("Number of bytes in data for {:?} is {}",std::any::type_name::<M>(),val_vec.len());
        let packet = GenericMsg {
            msg_type: MsgType::SET,
            timestamp: Utc::now().to_string(),
            name: self.name.to_string(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: val_vec.to_vec(),
        };
        // info!("The Node's packet to send looks like: {:?}",&packet);

        let packet_as_bytes: Vec<u8> = to_allocvec(&packet).unwrap();
        // info!("Node is publishing: {:?}",&packet_as_bytes);

        let topic = &self.topic;
        let stream = &mut self.stream.as_ref().unwrap();
        // This socket should actually be owned by the Node itself
        // let socket = UdpSocket::bind("127.0.0.1:25001");

        let _result = self.runtime.block_on(async {
            // TO_DO: Both Node and Host sockets needs to be part of the NodeConfig
            // This socket should actually be owned by the Node itself
            let socket = UdpSocket::bind("127.0.0.1:25001").await.unwrap();
            let len = socket
                .send_to(&packet_as_bytes, "127.0.0.1:25000")
                .await
                .unwrap();
        });

        Ok(())
    }

    /// Request data from host on Node's assigned topic
    #[tracing::instrument]
    pub fn request(&self) -> Result<T, postcard::Error> {
        let packet = GenericMsg {
            msg_type: MsgType::GET,
            timestamp: Utc::now().to_string(),
            name: self.name.to_string(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: Vec::new(),
        };
        // info!("{:?}",&packet);

        let packet_as_bytes: Vec<u8> = to_allocvec(&packet).unwrap();

        let stream = &mut self.stream.as_ref().unwrap();

        self.runtime.block_on(async {
            send_msg(stream, packet_as_bytes).await.unwrap();
            match await_response::<T>(stream, 4096).await {
                Ok(reply) => {
                    let data = from_bytes::<T>(&reply.data).unwrap();
                    Ok(data)
                }
                Err(e) => *Box::new(Err(e)),
            }
        })
    }

    /// Re-construct a Node's initial configuration to make it easy to re-build similar Node's
    pub fn rebuild_config(&self) -> NodeConfig<T> {
        let topic = self.topic.clone();
        let host_addr = match &self.stream {
            Some(stream) => stream.peer_addr().unwrap(),
            None => SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
        };

        NodeConfig {
            host_addr,
            topic: Some(topic),
            name: self.name.clone(),
            phantom: PhantomData,
        }
    }
}
