use std::net::UdpSocket;

use std::error::Error;
use std::result::Result;

use postcard::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::msg::*;

#[derive(Debug)]
pub struct Node {
    socket: Option<UdpSocket>,
    topic_name: String,
}

impl Node {
    pub fn new(topic_name: &str) -> Self {
        let mut socket_num = 25_001;
        let mut node_socket: Option<UdpSocket> = None;
        while node_socket.is_none() {
            let node_port = "127.0.0.1:".to_owned() + &socket_num.to_string();
            println!("Trying to assign to: {:?}", &node_port);
            node_socket = match UdpSocket::bind(node_port) {
                Ok(socket) => Some(socket),
                _ => None,
            };
            if (25_001..25_255).contains(&socket_num) {
                socket_num += 1;
            } else if socket_num > 25_255 {
                panic!("Couldn't assign node to socket over range");
            }
        }

        Node {
            socket: node_socket,
            topic_name: topic_name.into(),
        }
    }

    pub fn publish<T: Serialize + DeserializeOwned + std::fmt::Debug>(
        &self,
        val: T,
    ) -> Result<(), Box<dyn Error>> {
        /*
        println!(
            "The passed value is: {:?}, of type {:?}",
            val,
            std::any::type_name::<T>().to_string()
        );

        println!(
            "Our node topic's name is: {}, with length {}",
            std::str::from_utf8(&name_vec).unwrap(),
            name_length
        );*/

        let packet = RhizaMsg {
            name: self.topic_name.to_string(),
            type_info: std::any::type_name::<T>().to_string(),
            data: Some(val),
        };

        let packet_as_bytes: heapless::Vec<u8, 1024> = to_vec(&packet).unwrap();

        self.socket
            .as_ref()
            .unwrap()
            .send_to(&packet_as_bytes, "127.0.0.1:25000")?;

        Ok(())
    }

    /*
    pub fn request<T: Serialize + DeserializeOwned + std::fmt::Debug>(&self, topic_name: &str) -> Result<T, Box<dyn Error>> {

        Ok(())
    } 
    */
}
