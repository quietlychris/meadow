use std::net::IpAddr;

// use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
// use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;

use std::sync::Arc;
use tokio::sync::Mutex;
// use tokio::time::{sleep, Duration};

use std::error::Error;
use std::marker::PhantomData;
use std::result::Result;

use postcard::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::msg::*;

// pub trait Retrievable: Serialize + DeserializeOwned + std::fmt::Debug {}

// TO_DO: Make rx,tx OwnedRead/WriteHalf or normal Read/WriteHalf?
#[derive(Debug)]
pub struct Node<T: Serialize + DeserializeOwned + std::fmt::Debug> {
    stream: Option<TcpStream>,
    topic_name: String,
    phantom: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned + std::fmt::Debug> Node<T> {
    pub fn default(topic_name: &str) -> Self {
        let socket_num = 25_001;
        let node_socket: Option<TcpStream> = None;

        let node: Node<T> = Node {
            stream: None,
            topic_name: topic_name.into(),
            phantom: PhantomData,
        };

        node
    }

    pub async fn interface(&mut self, interface_name: &str) -> Result<(), Box<dyn Error>> {
        let ip = get_ip(interface_name).unwrap();
        dbg!(ip);
        //let mut socket_num = 25_001;
        while self.stream.is_none() {
            match TcpStream::connect("127.0.0.1:25000").await {
                Ok(stream) => {
                    println!("- Assigning to {:?}", &stream.local_addr()?);
                    self.stream = Some(stream);
                    println!("\t - Successfully assigned")
                }
                Err(e) => println!("Error: {:?}", e),
            };
        }

        Ok(())
    }

    pub async fn publish(&mut self, val: T) -> Result<(), Box<dyn Error>> {
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

        //println!("val: {:?}", &val);
        let packet = RhizaMsg {
            msg_type: Msg::SET,
            name: self.topic_name.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: val,
        };
        // println!("{:?}", &packet);

        let packet_as_bytes: heapless::Vec<u8, 4096> = to_vec(&packet).unwrap();

        let stream = self.stream.as_ref().unwrap();
        // let mut stream = self.stream.as_ref().unwrap().lock().await;
        // println!("Writing from node");
        loop {
            stream.writable().await?;
            match stream.try_write(&packet_as_bytes) {
                Ok(n) => {
                    // println!("Successfully wrote {} bytes to host", n);
                    break;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }

    pub async fn request(&mut self) -> Result<T, Box<dyn Error>> {
        let packet = GenericRhizaMsg {
            msg_type: Msg::GET,
            name: self.topic_name.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: Vec::new(),
        };
        // println!("{:?}", &packet);

        let packet_as_bytes: heapless::Vec<u8, 4096> = to_vec(&packet).unwrap();

        let stream = self.stream.as_ref().unwrap();
        //let stream = stream.as_ref().unwrap().lock().await;

        stream.writable().await?;
        
            // Write the request
            loop {
                match stream.try_write(&packet_as_bytes) {
                    Ok(n) => {
                        // println!("Successfully wrote {}-byte request to host", n);
                        break;
                    }
                    Err(e)  => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {

                        }
                        continue;
                    }
                }
            }

            // Wait for the response
            //stream.readable().await?;
            let mut buf = [0u8; 4096];
            loop {
                stream.readable().await?;
                match stream.try_read(&mut buf) {
                    Ok(0) => continue,
                    Ok(n) => {
                        let bytes = &buf[..n];
                        let msg: RhizaMsg<T> = from_bytes(&bytes).unwrap();
                        return Ok(msg.data);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // println!("Got WouldBlock error");
                        continue;
                    }
                    Err(e) => {
                        //return Err(e.into());
                        continue;
                    }
                }
            }
        
    }
}

fn get_ip(interface_name: &str) -> Result<String, Box<dyn Error>> {
    let interface = pnet::datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .expect(&format!(
            "IP address for interface \"{}\" does not exist or is not up",
            interface_name
        ));

    // dbg!(&interface);
    let source_ip = interface
        .ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .map(|ip| match ip.ip() {
            IpAddr::V4(ip) => ip,
            _ => unreachable!(),
        })
        .expect(&format!(
            "IP address for interface {} does not exist or is not up",
            interface_name
        ))
        .to_string();

    Ok(source_ip)
}
