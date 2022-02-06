extern crate alloc;

use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use tracing::*;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use std::error::Error;
use std::marker::{PhantomData, Sync};
use std::ops::Deref;
use std::result::Result;
use std::sync::Arc;

use alloc::vec::Vec;
use postcard::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::msg::*;
use chrono::Utc;

use std::fmt::Debug;
pub trait Message: Serialize + DeserializeOwned + Debug + Sync + Send + Clone {}
impl<T> Message for T where T: Serialize + DeserializeOwned + Debug + Sync + Send + Clone {}

/// Configuration of strongly-typed Node
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NodeConfig<T: Message> {
    host_addr: SocketAddr,
    name: String,
    topic: Option<String>,
    phantom: PhantomData<T>,
}

#[derive(Debug)]
pub struct Idle;
#[derive(Debug)]
pub struct Active;
#[derive(Debug)]
pub struct Subscription;

/// A named, strongly-typed Node capable of publish/request on Host
#[derive(Debug)]
pub struct Node<State, T: Message> {
    __state: PhantomData<State>,
    phantom: PhantomData<T>,
    runtime: Runtime,
    stream: Option<TcpStream>,
    name: String,
    topic: String,
    host_addr: SocketAddr,
    pub subscription_data: Arc<TokioMutex<Option<T>>>,
    task_subscribe: Option<JoinHandle<()>>,
}

impl<T: Message> From<Node<Idle, T>> for Node<Active, T> {
    fn from(node: Node<Idle, T>) -> Self {
        Self {
            __state: PhantomData,
            phantom: PhantomData,
            runtime: node.runtime,
            stream: node.stream,
            name: node.name,
            topic: node.topic,
            host_addr: node.host_addr,
            subscription_data: node.subscription_data,
            task_subscribe: None,
        }
    }
}

impl<T: Message> From<Node<Idle, T>> for Node<Subscription, T> {
    fn from(node: Node<Idle, T>) -> Self {
        Self {
            __state: PhantomData,
            phantom: PhantomData,
            runtime: node.runtime,
            stream: node.stream,
            name: node.name,
            topic: node.topic,
            host_addr: node.host_addr,
            subscription_data: node.subscription_data,
            task_subscribe: None,
        }
    }
}

impl<T: Message> NodeConfig<T> {
    /// Create a named, strongly-typed Node without an assigned topic
    pub fn new(name: impl Into<String>) -> NodeConfig<T> {
        NodeConfig {
            name: name.into(),
            topic: None,
            host_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 25_000),
            phantom: PhantomData,
        }
    }

    /// Convenience method for re-setting the name of the Node to be generated
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set topic of the generated Node
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Assign an address for the Host the Node will attempt to connect with
    pub fn host_addr(mut self, host_addr: impl Into<SocketAddr>) -> Self {
        self.host_addr = host_addr.into();
        self
    }

    /// Construct a Node from the specified configuration
    pub fn build(self) -> Result<Node<Idle, T>, Box<dyn Error>> {
        let runtime = tokio::runtime::Runtime::new()?;

        let topic = match self.topic {
            Some(topic) => topic,
            None => panic!("Nodes must have an assigned topic to be built"),
        };

        Ok(Node::<Idle, T> {
            __state: PhantomData,
            phantom: PhantomData,
            runtime,
            stream: None,
            host_addr: self.host_addr,
            name: self.name,
            topic,
            subscription_data: Arc::new(TokioMutex::new(None)),
            task_subscribe: None,
        })
    }
}

impl<T: Message + 'static> Node<Idle, T> {
    /// Attempt connection from the Node to the Host located at the specified address
    #[tracing::instrument]
    pub fn connect(mut self) -> Result<Node<Active, T>, Box<dyn Error>> {
        // let ip = crate::get_ip(interface).unwrap();
        // dbg!(ip);
        let addr = self.host_addr;
        let topic = self.topic.clone();

        let stream = self.runtime.block_on(async move {
            let stream = attempt_connection(addr).await.unwrap();
            let stream = handshake(stream, topic).await.unwrap();
            stream
        });
        self.stream = Some(stream);

        Ok(Node::<Active, T>::from(self))
    }

    #[tracing::instrument]
    pub fn subscribe(mut self, rate: Duration) -> Result<Node<Subscription, T>, Box<dyn Error>> {
        let name = self.name.clone() + "_SUBSCRIPTION";
        let addr = self.host_addr;
        let topic = self.topic.clone();

        let subscription_data: Arc<TokioMutex<Option<T>>> = Arc::new(TokioMutex::new(None));
        let data = Arc::clone(&subscription_data);

        let task_subscribe = self.runtime.spawn(async move {
            //let mut subscription_node = subscription_node.connect().unwrap();
            let stream = attempt_connection(addr).await.unwrap();
            let stream = handshake(stream, topic.clone()).await.unwrap();
            let packet = GenericMsg {
                msg_type: MsgType::GET,
                timestamp: Utc::now().to_string(),
                name: name.clone(),
                topic: topic.clone(),
                data_type: std::any::type_name::<T>().to_string(),
                data: Vec::new(),
            };
            // info!("{:?}",&packet);

            loop {
                let packet_as_bytes: Vec<u8> = to_allocvec(&packet).unwrap();
                send_request(&mut &stream, packet_as_bytes).await.unwrap();
                let reply = match await_response::<T>(&mut &stream, name.clone(), 4096).await {
                    Ok(val) => val,
                    Err(e) => {
                        error!("subscription error: {}", e);
                        continue;
                    }
                };
                let mut data = data.lock().await;
                *data = Some(reply);
                sleep(rate).await;
            }
        });
        self.task_subscribe = Some(task_subscribe);
        println!("spawned subscription task");

        let mut subscription_node = Node::<Subscription, T>::from(self);
        subscription_node.subscription_data = subscription_data;

        Ok(subscription_node)
    }
}

async fn attempt_connection(host_addr: SocketAddr) -> Result<TcpStream, Box<dyn Error>> {
    let mut connection_attempts = 0;
    let mut stream: Option<TcpStream> = None;
    while connection_attempts < 5 {
        match TcpStream::connect(host_addr).await {
            Ok(my_stream) => {
                stream = Some(my_stream);
                break;
            }
            Err(e) => {
                connection_attempts += 1;
                sleep(Duration::from_millis(1_000)).await;
                warn!("{:?}", e);
                // println!("Error: {:?}", e)
            }
        }
    }

    let stream = stream.unwrap();
    Ok(stream)
}

async fn handshake(stream: TcpStream, topic: String) -> Result<TcpStream, Box<dyn Error>> {
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
                    error!("NODE handshake error: {:?}", e);
                }
            }
        }
    }
    info!("{}: Successfully connected to host", topic);
    // TO_DO: Is there a better way to do this?
    // Pause after connection to avoid accidentally including published data in initial handshake
    sleep(Duration::from_millis(20)).await;

    Ok(stream)
}

impl<T: Message + 'static> Node<Subscription, T> {
    // Should actually return a <T>
    pub fn get_subscribed_data(&self) -> Result<Option<T>, Box<dyn Error>> {
        let data = self.subscription_data.clone();
        let result = self.runtime.block_on(async {
            let data = data.lock().await;
            data.deref().clone()
        });
        Ok(result)
    }
}

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
                        let _msg: Result<String, Box<dyn Error>> = match from_bytes(bytes) {
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

    /// Request data from host on Node's assigned topic
    // TO_DO: Should this return a StdError or postcard::Error?
    #[tracing::instrument]
    pub fn request(&self) -> Result<T, Box<dyn Error>> {
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
        let name = self.name.to_string();

        self.runtime.block_on(async {
            send_request(stream, packet_as_bytes).await.unwrap();
            await_response::<T>(stream, name, 4096).await
        })
    }

    /// Re-construct a Node's initial configuration to make it easy to re-build similar Node's
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

async fn send_request(
    stream: &mut &TcpStream,
    packet_as_bytes: Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    stream.writable().await.unwrap();

    // Write the request
    // TO_DO: This should be a loop with a maximum number of attempts
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
    Ok(())
}

async fn await_response<T: Message>(
    stream: &mut &TcpStream,
    name: String,
    _max_buffer_size: usize, // TO_DO: This should be configurable
) -> Result<T, Box<dyn Error>> {
    // Read the requested data into a buffer
    let mut buf = [0u8; 4096];
    // let mut buf = Vec::with_capacity(max_buffer_size);

    loop {
        stream.readable().await.unwrap();
        match stream.try_read(&mut buf) {
            Ok(0) => continue,
            Ok(n) => {
                let bytes = &buf[..n];
                let _msg: Result<GenericMsg, Box<dyn Error>> = match from_bytes(bytes) {
                    Ok(msg) => {
                        let msg: GenericMsg = msg;

                        // TO_DO: Put logging info like this behind a feature flag
                        // let delta = (Utc::now() - msg.timestamp.parse::<DateTime<Utc>>()?).to_std()?.as_micros();
                        // println!("The time difference between msg tx/rx is: {} us",delta);
                        // info!("Node has received msg data: {:?}",&msg.data);

                        match from_bytes::<T>(&msg.data) {
                            Ok(data) => return Ok(data),
                            Err(e) => {
                                println!("Error: {:?}, &msg.data: {:?}", e, &msg.data);
                                println!("Expected type: {}", std::any::type_name::<T>());
                                return Err(Box::new(e));
                            }
                        }
                    }
                    Err(e) => {
                        error!("{}: {:?}", name, &e);
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
}
