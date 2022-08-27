use crate::Error;
use crate::*;

use chrono::Utc;

use postcard::*;
use std::result::Result;
use tracing::*;

impl<T: Message + 'static> Node<Active, T> {
    // TO_DO: The error handling in the async blocks need to be improved
    /// Send data to host on Node's assigned topic using Msg<T> packet
    #[tracing::instrument]
    pub fn publish(&self, val: T) -> Result<(), Error> {
        let val_vec: Vec<u8> = match to_allocvec(&val) {
            Ok(val_vec) => val_vec,
            Err(_e) => return Err(Error::Serialization),
        };

        // println!("Number of bytes in data for {:?} is {}",std::any::type_name::<M>(),val_vec.len());
        let packet = GenericMsg {
            msg_type: MsgType::SET,
            timestamp: Utc::now(),
            name: self.name.to_string(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: val_vec.to_vec(),
        };
        // info!("The Node's packet to send looks like: {:?}",&packet);

        let packet_as_bytes: Vec<u8> = match to_allocvec(&packet) {
            Ok(packet) => packet,
            Err(_e) => return Err(Error::Serialization),
        };
        // info!("Node is publishing: {:?}",&packet_as_bytes);

        let mut stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        self.runtime.block_on(async {
            send_msg(&mut stream, packet_as_bytes).await.unwrap();

            // Wait for the publish acknowledgement
            let mut buf = vec![0u8; 1024];
            loop {
                stream.readable().await.unwrap();
                match stream.try_read(&mut buf) {
                    Ok(0) => continue,
                    Ok(n) => {
                        let bytes = &buf[..n];
                        // TO_DO: This error handling is not great
                        // There should be some kind of check on the Host-sent KV-storage ack message
                        // This ack message should be a Result/Option for success/failure, not the
                        // String that is currently used
                        let _msg = if let Ok(_msg) = std::str::from_utf8(bytes) {
                            // dbg!(msg.to_string());
                        };

                        break;
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {}
                        continue;
                    }
                }
            }
        });

        Ok(())
    }

    #[tracing::instrument]
    pub fn publish_udp(&self, val: T) -> Result<(), Error> {
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

        let socket = match self.socket.as_ref() {
            Some(socket) => socket,
            None => return Err(Error::AccessSocket),
        };

        self.runtime.block_on(async {
            match socket
                .send_to(&packet_as_bytes, self.cfg.udp.host_addr)
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

    /// Request data from host on Node's assigned topic
    #[tracing::instrument]
    pub fn request(&self) -> Result<T, Error> {
        let packet = GenericMsg {
            msg_type: MsgType::GET,
            timestamp: Utc::now(),
            name: self.name.to_string(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: Vec::new(),
        };
        // info!("{:?}",&packet);

        let packet_as_bytes: Vec<u8> = match to_allocvec(&packet) {
            Ok(packet) => packet,
            Err(_e) => return Err(Error::Serialization),
        };

        let mut stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        self.runtime.block_on(async {
            send_msg(&mut stream, packet_as_bytes).await.unwrap();
            match await_response::<T>(&mut stream, self.cfg.tcp.max_buffer_size).await {
                Ok(reply) => {
                    match from_bytes::<T>(&reply.data) {
                        Ok(data) => Ok(data),
                        Err(_e) => Err(Error::Deserialization),
                    }
                    // data_result
                }
                Err(_e) => Err(Error::BadResponse),
            }
        })
    }
}
