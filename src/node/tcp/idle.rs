extern crate alloc;
use crate::Error;
use crate::*;

use crate::node::network_config::Tcp;
use crate::node::*;

use tokio::net::UdpSocket;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

use tracing::*;

use std::net::SocketAddr;
use std::result::Result;
use std::sync::Arc;

use alloc::vec::Vec;
use postcard::*;
use std::marker::PhantomData;

#[cfg(feature = "quic")]
use quinn::Endpoint;

use crate::msg::*;
use chrono::Utc;

impl<T: Message> From<Node<Tcp, Idle, T>> for Node<Tcp, Active, T> {
    fn from(node: Node<Tcp, Idle, T>) -> Self {
        Self {
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
            stream: node.stream,
            topic: node.topic,
            socket: node.socket,
            buffer: node.buffer,
            #[cfg(feature = "quic")]
            endpoint: node.endpoint,
            #[cfg(feature = "quic")]
            connection: node.connection,
            subscription_data: node.subscription_data,
            task_subscribe: None,
        }
    }
}

impl<T: Message> From<Node<Tcp, Idle, T>> for Node<Tcp, Subscription, T> {
    fn from(node: Node<Tcp, Idle, T>) -> Self {
        Self {
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
            stream: node.stream,
            topic: node.topic,
            socket: node.socket,
            buffer: node.buffer,
            #[cfg(feature = "quic")]
            endpoint: node.endpoint,
            #[cfg(feature = "quic")]
            connection: node.connection,
            subscription_data: node.subscription_data,
            task_subscribe: None,
        }
    }
}

impl<T: Message + 'static> Node<Tcp, Idle, T> {
    /// Attempt connection from the Node to the Host located at the specified address
    #[tracing::instrument(skip_all)]
    pub fn activate(mut self) -> Result<Node<Tcp, Active, T>, Error> {
        let addr = self.cfg.network_cfg.host_addr;
        let topic = self.topic.clone();

        let stream = self.runtime.block_on(async move {
            match try_connection(addr).await {
                Ok(stream) => match handshake(stream, topic).await {
                    Ok(stream) => Ok(stream),
                    Err(e) => {
                        error!("{:?}", e);
                        Err(Error::Handshake)
                    }
                },
                Err(e) => {
                    error!("{:?}", e);
                    Err(Error::StreamConnection)
                }
            }
        });
        match stream {
            Ok(stream) => {
                debug!(
                    "Established Node<=>Host TCP stream: {:?}",
                    stream.local_addr()
                );
                self.stream = Some(stream)
            }
            Err(e) => return Err(e),
        };

        Ok(Node::<Tcp, Active, T>::from(self))
    }

    #[tracing::instrument]
    pub fn subscribe(mut self, rate: Duration) -> Result<Node<Tcp, Subscription, T>, Error> {
        let addr = self.cfg.network_cfg.host_addr;
        let topic = self.topic.clone();

        let subscription_data: Arc<TokioMutex<Option<Msg<T>>>> = Arc::new(TokioMutex::new(None));
        let data = Arc::clone(&subscription_data);

        let buffer = self.buffer.clone();

        let task_subscribe = self.runtime.spawn(async move {
            let stream = match try_connection(addr).await {
                Ok(stream) => match handshake(stream, topic.clone()).await {
                    Ok(stream) => Ok(stream),
                    Err(e) => {
                        error!("{:?}", e);
                        Err(Error::Handshake)
                    }
                },
                Err(e) => {
                    error!("{:?}", e);
                    Err(Error::StreamConnection)
                }
            }
            .unwrap();
            debug!("Successfully subscribed to Host");

            let packet = GenericMsg {
                msg_type: MsgType::GET,
                timestamp: Utc::now(),
                topic: topic.clone(),
                data_type: std::any::type_name::<T>().to_string(),
                data: Vec::new(),
            };

            let mut buffer = buffer.lock().await;
            loop {
                let packet_as_bytes: Vec<u8> = to_allocvec(&packet).unwrap();
                send_msg(&stream, packet_as_bytes).await.unwrap();
                let msg = match await_response::<T>(&stream, &mut buffer).await {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Subscription Error: {}", e);
                        continue;
                    }
                };
                let delta = Utc::now() - msg.timestamp;
                // println!("The time difference between msg tx/rx is: {} us",delta);
                if delta <= chrono::Duration::zero() {
                    // println!("Data is not newer, skipping to next subscription iteration");
                    continue;
                }

                let mut data = data.lock().await;

                *data = Some(msg);
                sleep(rate).await;
            }
        });
        self.task_subscribe = Some(task_subscribe);

        let mut subscription_node = Node::<Tcp, Subscription, T>::from(self);
        subscription_node.subscription_data = subscription_data;

        Ok(subscription_node)
    }
}
