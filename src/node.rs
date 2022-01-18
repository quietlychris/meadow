use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

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
    name: String,
    phantom: PhantomData<T>,
}

#[derive(Debug)]
pub struct Node<T: Message> {
    runtime: Runtime,
    stream: Option<TcpStream>,
    name: String,
    host_addr: SocketAddr,
    phantom: PhantomData<T>,
}

impl<T: Message> NodeConfig<T> {
    pub fn new(name: impl Into<String>) -> NodeConfig<T> {
        NodeConfig {
            name: name.into(),
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            phantom: PhantomData,
        }
    }

    pub fn host_addr(mut self, host_addr: impl Into<SocketAddr>) -> Self {
        self.host_addr = host_addr.into();
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn build(self) -> Result<Node<T>, Box<dyn Error>> {
        let runtime = tokio::runtime::Runtime::new()?;

        Ok(Node::<T> {
            runtime,
            stream: None,
            host_addr: self.host_addr,
            name: self.name,
            phantom: PhantomData,
        })
    }
}

impl<T: Message + 'static> Node<T> {
    pub fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        // let ip = crate::get_ip(interface).unwrap();
        // dbg!(ip);
        //let mut socket_num = 25_001;

        // let stream: Arc<Mutex<Option<TcpStream>>> = Arc::new(Mutex::new(None));
        let host_addr = self.host_addr;
        let name = self.name.clone();
        let stream = &mut self.stream;

        self.runtime.block_on(async {
            // println!("hello!");
            match TcpStream::connect(host_addr).await {
                Ok(my_stream) => {
                    *stream = Some(my_stream);
                }
                Err(e) => println!("Error: {:?}", e),
            }

            sleep(Duration::from_millis(2)).await;

            let mut stream = stream.as_ref().unwrap();
            loop {
                stream.writable().await.unwrap();
                match stream.try_write(name.as_bytes()) {
                    Ok(_n) => {
                        // println!("Successfully wrote {} bytes to host", n);
                        break;
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                        } else {
                            println!("Handshake error: {:?}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    // TO_DO: The error handling in the async blocks need to be improved
    // Type M of published message is not necessarily the same as Type T assigned to the Node
    pub fn publish_to<M: Message>(
        &mut self,
        name: impl Into<String>,
        val: M,
    ) -> Result<(), Box<dyn Error>> {
        let packet = RhizaMsg {
            msg_type: Msg::SET,
            name: self.name.to_string(),
            data_type: std::any::type_name::<M>().to_string(),
            data: val,
        };

        let packet_as_bytes: heapless::Vec<u8, 4096> = to_vec(&packet).unwrap();

        //let stream = self.stream.lock().unwrap(); // .as_ref().unwrap();
        let stream = &mut self.stream.as_ref().unwrap();

        let result = self.runtime.block_on(async {
            loop {
                stream.writable().await.unwrap();
                match stream.try_write(&packet_as_bytes) {
                    Ok(_n) => {
                        // println!("Successfully wrote {} bytes to host", n);
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
        });

        Ok(())
    }

    pub fn request<M: Message>(
        &mut self,
        name: impl Into<String>,
    ) -> Result<M, Box<postcard::Error>> {
        let packet = GenericRhizaMsg {
            msg_type: Msg::GET,
            name: self.name.to_string(),
            data_type: std::any::type_name::<M>().to_string(),
            data: Vec::new(),
        };
        // println!("{:?}", &packet);

        let packet_as_bytes: heapless::Vec<u8, 4096> = to_vec(&packet).unwrap();

        let stream = &mut self.stream.as_ref().unwrap();
        //let stream = stream.as_ref().unwrap().lock().await;

        self.runtime.block_on(async {
            stream.writable().await.unwrap();

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
                stream.readable().await.unwrap();
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
        })
    }

    pub fn rebuild_config(&self) -> NodeConfig<T> {
        let name = self.name.clone();
        dbg!(&name);
        let host_addr = match &self.stream {
            Some(stream) => stream.peer_addr().unwrap(),
            None => SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
        };
        dbg!(&host_addr);

        NodeConfig {
            host_addr,
            name,
            phantom: PhantomData,
        }
    }
}
