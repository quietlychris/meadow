extern crate alloc;

use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

use tracing::*;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use std::error::Error;
use std::marker::PhantomData;
use std::result::Result;

use postcard::*;
use alloc::vec::Vec;
use serde::{de::DeserializeOwned, Serialize};

use crate::msg::*;

use std::fmt::{Debug, Display};
pub trait Message: Serialize + DeserializeOwned + Debug + Send {}
impl<T> Message for T where T: Serialize + DeserializeOwned + Debug + Send {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NodeConfig<T: Message> {
    host_addr: SocketAddr,
    name: String,
    topic: Option<String>,
    phantom: PhantomData<T>,
}

#[derive(Debug)]
pub struct Node<T: Message> {
    runtime: Runtime,
    stream: Option<TcpStream>,
    name: String,
    topic: String,
    host_addr: SocketAddr,
    phantom: PhantomData<T>,
}

impl<T: Message> NodeConfig<T> {
    pub fn new(name: impl Into<String>) -> NodeConfig<T> {
        NodeConfig {
            name: name.into(),
            topic: None,
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            phantom: PhantomData,
        }
    }

    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    pub fn host_addr(mut self, host_addr: impl Into<SocketAddr>) -> Self {
        self.host_addr = host_addr.into();
        self
    }

    pub fn build(self) -> Result<Node<T>, Box<dyn Error>> {
        let runtime = tokio::runtime::Runtime::new()?;
        
        let topic = match self.topic {
            Some(topic) => topic,
            None => panic!("Nodes must have an assigned topic to be built")
        };

        Ok(Node::<T> {
            runtime,
            stream: None,
            host_addr: self.host_addr,
            name: self.name,
            topic,
            phantom: PhantomData,
        })
    }
}

impl<T: Message + 'static> Node<T> {
    
    #[tracing::instrument]
    pub fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        // let ip = crate::get_ip(interface).unwrap();
        // dbg!(ip);
        //let mut socket_num = 25_001;

        // let stream: Arc<Mutex<Option<TcpStream>>> = Arc::new(Mutex::new(None));
        let host_addr = self.host_addr;
        let topic = self.topic.clone();
        let stream = &mut self.stream;

        self.runtime.block_on(async {
            // println!("hello!");
            let mut connection_attempts = 0;
            while connection_attempts < 5 {
                match TcpStream::connect(host_addr).await {
                    Ok(my_stream) => {
                        *stream = Some(my_stream);
                        break;
                    }
                    Err(e) => {
                        connection_attempts += 1;
                        sleep(Duration::from_millis(1_000)).await;
                        warn!("{:?}",e);
                        // println!("Error: {:?}", e)
                    }
                }
            }

            sleep(Duration::from_millis(2)).await;

            let mut stream = stream.as_ref().unwrap();
            loop {
                stream.writable().await.unwrap();
                match stream.try_write(topic.as_bytes()) {
                    Ok(_n) => {
                        // println!("Successfully wrote {} bytes to host", n);
                        info!("{}: Wrote {} bytes to host", topic, _n);
                        break;
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                        } else {
                            // println!("Handshake error: {:?}", e);
                            error!("NODE handshake error: {:?}",e);
                        }
                    }
                }
            }
            info!("{}: Successfully connected to host", topic);
            // TO_DO: Is there a better way to do this?
            // Pause after connection to avoid accidentally including published data in initial handshake
            sleep(Duration::from_millis(20)).await;
        });

        Ok(())
    }

    // TO_DO: The error handling in the async blocks need to be improved
    // Type M of published message is not necessarily the same as Type T assigned to the Node
    #[tracing::instrument]
    pub fn publish(
        &mut self,
        // topic: impl Into<String> + Debug + Display,
        val: T,
    ) -> Result<(), Box<dyn Error>> {

        // let val_vec: heapless::Vec<u8, 4096> = to_vec(&val).unwrap();
        let val_vec: Vec<u8> = to_allocvec(&val).unwrap();

        // println!("Number of bytes in data for {:?} is {}",std::any::type_name::<M>(),val_vec.len());
        let packet = GenericRhizaMsg {
            msg_type: Msg::SET,
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

        let result = self.runtime.block_on(async {
            loop {
                stream.writable().await.unwrap();
                match stream.try_write(&packet_as_bytes) {
                    Ok(_n) => {
                        // println!("Successfully wrote {} bytes to host", n);
                        info!(
                            "{}: Successfully wrote {} bytes to host",
                            topic.to_string(),
                            _n
                        );
                        break;
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {}
                        continue;
                    }
                }
            }

            // Wait for the publish acknowledgement
            //stream.readable().await?;
            let mut buf = [0u8; 4096];
            loop {
                stream.readable().await.unwrap();
                match stream.try_read(&mut buf) {
                    Ok(0) => continue,
                    Ok(n) => {
                        let bytes = &buf[..n];
                        let msg: Result<String, Box<dyn Error>> = match from_bytes(bytes) {
                            Ok(ack) => {
                                return Ok(ack);
                            }
                            Err(e) => {
                                error!("{}: {:?}", topic, &e);
                                return Err(Box::new(e));
                            }
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
        });

        Ok(())
    }

    #[tracing::instrument]
    pub fn request(
        &mut self,
        // topic: impl Into<String> + Debug + Display,
    ) -> Result<T, Box<postcard::Error>> {
        let packet = GenericRhizaMsg {
            msg_type: Msg::GET,
            name: self.name.to_string(),
            topic: self.topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: Vec::new(),
        };
        // info!("{:?}",&packet);

        let packet_as_bytes: Vec<u8> = to_allocvec(&packet).unwrap();

        let stream = &mut self.stream.as_ref().unwrap();

        self.runtime.block_on(async {
            stream.writable().await.unwrap();

            // Write the request
            loop {
                match stream.try_write(&packet_as_bytes) {
                    Ok(_n) => {
                        // info!("Node successfully wrote {}-byte request to host",n);
                        break;
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {}
                        continue;
                    }
                }
            }

            let mut buf = [0u8; 4096];
            loop {
                stream.readable().await.unwrap();
                match stream.try_read(&mut buf) {
                    Ok(0) => continue,
                    Ok(n) => {
                        let bytes = &buf[..n];
                        let msg: Result<GenericRhizaMsg, Box<dyn Error>> = match from_bytes(bytes) {
                            Ok(msg) => {
                                let msg: GenericRhizaMsg = msg;
                                // info!("Node has received msg data: {:?}",&msg.data);

                                match from_bytes::<T>(&msg.data) {
                                    Ok(data) => return Ok(data),
                                    Err(e) => {
                                        println!("Error: {:?}, &msg.data: {:?}",e,&msg.data);
                                        println!("Expected type: {}",std::any::type_name::<T>());
                                        return Err(Box::new(e))
                                    }
                                }
                            }
                            Err(e) => {
                                error!("{}: {:?}", self.name.to_string(), &e);
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
        })
    }

    pub fn rebuild_config(&self) -> NodeConfig<T> {
        let topic = self.topic.clone();
        let host_addr = match &self.stream {
            Some(stream) => stream.peer_addr().unwrap(),
            None => SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
        };
        // dbg!(&host_addr);

        NodeConfig {
            host_addr,
            topic: Some(topic),
            name: self.name.clone(),
            phantom: PhantomData,
        }
    }
}
