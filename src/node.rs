use tokio::net::TcpStream;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use std::error::Error;
use std::marker::PhantomData;
use std::result::Result;

use postcard::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::msg::*;

use std::fmt::Debug;
pub trait Message: Serialize + DeserializeOwned + Debug + Send {}
impl<T> Message for T where T: Serialize + DeserializeOwned + Debug + Send {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NodeConfig<T: Message> {
    host_addr: SocketAddr,
    topic_name: String,
    phantom: PhantomData<T>,
}

#[derive(Debug)]
pub struct Node<T: Message> {
    stream: Option<TcpStream>,
    topic_name: String,
    host_addr: SocketAddr,
    phantom: PhantomData<T>,
}

impl<T: Message> NodeConfig<T> {
    pub fn new(topic_name: impl Into<String>) -> NodeConfig<T> {
        NodeConfig {
            topic_name: topic_name.into(),
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            phantom: PhantomData,
        }
    }

    pub fn host_addr(mut self, host_addr: impl Into<SocketAddr>) -> Self {
        self.host_addr = host_addr.into();
        self
    }

    pub fn topic_name(mut self, topic_name: impl Into<String>) -> Self {
        self.topic_name = topic_name.into();
        self
    }
}

impl<T: Message + 'static> Node<T> {
    pub fn from_config(cfg: NodeConfig<T>) -> Node<T> {
        Node::<T> {
            stream: None,
            host_addr: cfg.host_addr,
            topic_name: cfg.topic_name,
            phantom: PhantomData,
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        // let ip = crate::get_ip(interface).unwrap();
        // dbg!(ip);
        //let mut socket_num = 25_001;
        while self.stream.is_none() {
            match TcpStream::connect(self.host_addr).await {
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

    // Type M of published message is not necessarily the same as Type T assigned to the Node
    pub async fn publish_to<M: Message>(
        &mut self,
        topic_name: impl Into<String>,
        val: M,
    ) -> Result<(), Box<dyn Error>> {
        let packet = RhizaMsg {
            msg_type: Msg::SET,
            name: self.topic_name.to_string(),
            data_type: std::any::type_name::<M>().to_string(),
            data: val,
        };

        let packet_as_bytes: heapless::Vec<u8, 4096> = to_vec(&packet).unwrap();

        let stream = self.stream.as_ref().unwrap();
        loop {
            stream.writable().await?;
            match stream.try_write(&packet_as_bytes) {
                Ok(_n) => {
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

        // Wait for the publish acknowledgement
        //stream.readable().await?;
        let mut buf = [0u8; 4096];
        loop {
            stream.readable().await?;
            match stream.try_read(&mut buf) {
                Ok(0) => continue,
                Ok(n) => {
                    let bytes = &buf[..n];
                    let msg: Result<String, Box<dyn Error>> = match from_bytes(bytes) {
                        Ok(ack) => {
                            
                            return Ok(ack);
                        }
                        Err(e) => return Err(Box::new(e)),
                    };
                    // return Ok(msg.data);
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {}
                    continue;
                }
            }
        }


        Ok(())
    }

    pub async fn request<M: Message>(
        &mut self,
        topic_name: impl Into<String>,
    ) -> Result<M, Box<dyn Error>> {
        let packet = GenericRhizaMsg {
            msg_type: Msg::GET,
            name: self.topic_name.to_string(),
            data_type: std::any::type_name::<M>().to_string(),
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
                Ok(_n) => {
                    // println!("Successfully wrote {}-byte request to host", n);
                    break;
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {}
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
                    let msg: Result<RhizaMsg<M>, Box<dyn Error>> = match from_bytes(bytes) {
                        Ok(msg) => {
                            let msg: RhizaMsg<M> = msg;
                            return Ok(msg.data);
                        }
                        Err(e) => return Err(Box::new(e)),
                    };
                    // return Ok(msg.data);
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {}
                    continue;
                }
            }
        }
    }

    pub fn rebuild_config(&self) -> NodeConfig<T> {
        let topic_name = self.topic_name.clone();
        dbg!(&topic_name);
        let host_addr = match &self.stream {
            Some(stream) => stream.peer_addr().unwrap(),
            None => SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
        };
        dbg!(&host_addr);

        NodeConfig {
            host_addr,
            topic_name,
            phantom: PhantomData,
        }
    }
}
