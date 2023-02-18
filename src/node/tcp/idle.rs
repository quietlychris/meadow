extern crate alloc;
use crate::Error;
use crate::*;

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

// Quic
use quinn::Endpoint;

use crate::msg::*;
use chrono::Utc;

impl<T: Message> From<Node<Tcp, Idle, T>> for Node<Tcp, Active, T> {
    fn from(node: Node<Tcp, Idle, T>) -> Self {
        Self {
            //__interface: PhantomData,
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
            stream: node.stream,
            name: node.name,
            topic: node.topic,
            socket: node.socket,
            endpoint: node.endpoint,
            subscription_data: node.subscription_data,
            task_subscribe: None,
        }
    }
}

impl<T: Message> From<Node<Tcp, Idle, T>> for Node<Tcp, Subscription, T> {
    fn from(node: Node<Tcp, Idle, T>) -> Self {
        Self {
            //__interface: PhantomData,
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
            stream: node.stream,
            name: node.name,
            topic: node.topic,
            socket: node.socket,
            endpoint: node.endpoint,
            subscription_data: node.subscription_data,
            task_subscribe: None,
        }
    }
}

impl<T: Message + 'static> Node<Tcp, Idle, T> {
    /// Attempt connection from the Node to the Host located at the specified address
    //#[tracing::instrument(skip_all)]
    pub fn activate(mut self) -> Result<Node<Tcp, Active, T>, Error> {
        //if let Some(tcp_cfg) = &self.cfg.tcp {
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
                info!(
                    "Established Node<=>Host TCP stream: {:?}",
                    stream.local_addr()
                );
                self.stream = Some(stream)
            }
            Err(e) => return Err(e),
        };
        //}

        /*
        if let Some(_udp_cfg) = &self.cfg.udp {
            match self.runtime.block_on(async move {
                match UdpSocket::bind("[::]:0").await {
                    Ok(socket) => Ok(socket),
                    Err(_e) => Err(Error::AccessSocket),
                }
            }) {
                Ok(socket) => self.socket = Some(socket),
                Err(e) => return Err(e),
            };
        }

        if let Some(quic_cfg) = &self.cfg.quic {
            info!("Attempting QUIC connection");
            let endpoint = self.runtime.block_on(async move {
                // QUIC, needs to be done inside of a tokio context
                let client_cfg = generate_client_config_from_certs();
                let client_addr = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
                let mut endpoint = Endpoint::client(client_addr).unwrap();
                endpoint.set_default_client_config(client_cfg);
                endpoint
            });
            info!("{:?}", &endpoint.local_addr());
            self.endpoint = Some(endpoint);
        }
        */

        Ok(Node::<Tcp, Active, T>::from(self))
    }

    //#[tracing::instrument]
    pub fn subscribe(mut self, rate: Duration) -> Result<Node<Tcp, Subscription, T>, Error> {
        /*
        let tcp_cfg = match &self.cfg.tcp {
            Some(tcp_cfg) => tcp_cfg,
            None => {
                // TO_DO: We should have a more specific error code for this
                return Err(Error::AccessStream);
            }
        };
        */

        let name = self.name.clone();
        let addr = self.cfg.network_cfg.host_addr;
        let topic = self.topic.clone();

        let subscription_data: Arc<TokioMutex<Option<SubscriptionData<T>>>> =
            Arc::new(TokioMutex::new(None));
        let data = Arc::clone(&subscription_data);

        let max_buffer_size = self.cfg.network_cfg.max_buffer_size;
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
            info!("Successfully subscribed to Host");

            let packet = GenericMsg {
                msg_type: MsgType::GET,
                timestamp: Utc::now(),
                name: name.clone(),
                topic: topic.clone(),
                data_type: std::any::type_name::<T>().to_string(),
                data: Vec::new(),
            };
            // info!("{:?}",&packet);

            loop {
                let packet_as_bytes: Vec<u8> = to_allocvec(&packet).unwrap();
                send_msg(&mut &stream, packet_as_bytes).await.unwrap();
                let reply = match await_response::<T>(&mut &stream, max_buffer_size).await {
                    Ok(val) => val,
                    Err(e) => {
                        error!("Subscription Error: {}", e);
                        continue;
                    }
                };
                let delta = Utc::now() - reply.timestamp;
                // println!("The time difference between msg tx/rx is: {} us",delta);
                if delta <= chrono::Duration::zero() {
                    // println!("Data is not newer, skipping to next subscription iteration");
                    continue;
                }
                // info!("Node has received msg data: {:?}",&msg.data);
                let reply_data = match from_bytes::<T>(&reply.data) {
                    Ok(data) => data,
                    Err(e) => {
                        error!("{:?}", e);
                        continue;
                    }
                };
                let reply_sub_data = SubscriptionData {
                    data: reply_data,
                    timestamp: reply.timestamp,
                };
                let mut data = data.lock().await;

                *data = Some(reply_sub_data);
                sleep(rate).await;
            }
        });
        self.task_subscribe = Some(task_subscribe);

        let mut subscription_node = Node::<Tcp, Subscription, T>::from(self);
        subscription_node.subscription_data = subscription_data;

        Ok(subscription_node)
    }
}
