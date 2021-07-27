use std::net::UdpSocket;
use std::net::{IpAddr};

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

    pub fn default(topic_name: &str) -> Self {
        let mut socket_num = 25_001;
        let mut node_socket: Option<UdpSocket> = None;

        let mut node = Node {
            socket: None,
            topic_name: topic_name.into()
        };

        node.interface("lo").unwrap();
   
        node

    }

    pub fn interface(&mut self, interface_name: &str) -> Result<(), Box<dyn Error>> {
        let ip = get_ip(interface_name).unwrap();

        let mut socket_num = 25_001;
        while self.socket.is_none() {
            let node_port = ip.clone() + ":" + &socket_num.to_string();
            println!("Trying to assign to: {:?}", &node_port);
            self.socket = match UdpSocket::bind(node_port) {
                Ok(socket) => {
                    Some(socket)
                },
                _ => None,
            };
            if (25_001..25_255).contains(&socket_num) {
                socket_num += 1;
            } else if socket_num > 25_255 {
                panic!("Couldn't assign node to socket over range");
            }
        }
        println!("- Successfully assigned to {:?}",self.socket.as_ref().unwrap().local_addr()?);

        Ok(())

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

fn get_ip(interface_name: &str) -> Result<String, Box<dyn Error>> {

    let interface = pnet::datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .expect(&format!("IP address for interface \"{}\" does not exist or is not up",interface_name));

    // dbg!(&interface);
    let source_ip = interface
        .ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .map(|ip| match ip.ip() {
            IpAddr::V4(ip) => ip,
            _ => unreachable!(),
        })
        .expect(&format!("IP address for interface {} does not exist or is not up",interface_name))
        .to_string();

    Ok(source_ip)

}