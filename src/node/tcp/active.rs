use crate::Error;
use crate::*;

use chrono::Utc;

use postcard::*;
#[cfg(feature = "quic")]
use quinn::Connection as QuicConnection;
use std::result::Result;
use tracing::*;

use crate::node::network_config::{Interface, Tcp};

/// Tcp implements the Interface trait
impl Interface for Tcp {}

impl<T: Message + 'static> Node<Tcp, Active, T> {
    // TO_DO: The error handling in the async blocks need to be improved
    /// Send data to host on Node's assigned topic using `Msg<T>` packet
    #[tracing::instrument]
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

        let stream = match self.stream.as_ref() {
            Some(stream) => stream,
            None => return Err(Error::AccessStream),
        };

        self.runtime.block_on(async {
            crate::node::tcp::send_msg(stream, packet_as_bytes)
                .await
                .unwrap();

            // Wait for the publish acknowledgement
            let mut buf = vec![0u8; 1024];
            loop {
                stream.readable().await.unwrap();
                match stream.try_read(&mut buf) {
                    Ok(0) => continue,
                    Ok(n) => {
                        let bytes = &buf[..n];
                        // TO_DO: This error handling is not great
                        match from_bytes::<Result<(), Error>>(bytes) {
                            Err(e) => {
                                error!("{:?}", e);
                            }
                            Ok(result) => match result {
                                Ok(()) => (),
                                Err(e) => {
                                    error!("{:?}", e);
                                }
                            },
                        };

                        break;
                    }
                    Err(_e) => {
                        // if e.kind() == std::io::ErrorKind::WouldBlock {}
                        continue;
                    }
                }
            }
        });

        Ok(())
    }

    /// Request data from host on Node's assigned topic
    #[tracing::instrument]
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

        let packet_as_bytes: Vec<u8> = match to_allocvec(&packet) {
            Ok(packet) => packet,
            Err(_e) => return Err(Error::Serialization),
        };

        self.runtime.block_on(async {
            crate::node::tcp::send_msg(stream, packet_as_bytes)
                .await
                .unwrap();
            match crate::node::tcp::await_response::<T>(
                stream,
                self.cfg.network_cfg.max_buffer_size,
            )
            .await
            {
                Ok(msg) => Ok(msg),
                Err(_e) => Err(Error::Deserialization),
            }
        })
    }
}
