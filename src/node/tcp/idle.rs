extern crate alloc;
use crate::prelude::*;

use crate::node::network_config::{Nonblocking, Tcp};
use crate::node::*;

use tcp::try_connection;
use tokio::net::UdpSocket;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

use tracing::*;

use std::convert::TryInto;
use std::net::SocketAddr;
use std::result::Result;
use std::sync::Arc;

use alloc::vec::Vec;
use postcard::{from_bytes, to_allocvec};
use std::marker::PhantomData;

#[cfg(feature = "quic")]
use quinn::Endpoint;

use crate::msg::*;
use chrono::Utc;

impl<T: Message> From<Node<Nonblocking, Tcp, Idle, T>> for Node<Nonblocking, Tcp, Active, T> {
    fn from(node: Node<Nonblocking, Tcp, Idle, T>) -> Self {
        Self {
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
            rt_handle: node.rt_handle,
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

impl<T: Message> From<Node<Nonblocking, Tcp, Idle, T>> for Node<Nonblocking, Tcp, Subscription, T> {
    fn from(node: Node<Nonblocking, Tcp, Idle, T>) -> Self {
        Self {
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
            rt_handle: node.rt_handle,
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

use crate::node::tcp::handshake;
use tokio::net::TcpStream;

impl<T: Message + 'static> Node<Nonblocking, Tcp, Idle, T> {
    /// Attempt connection from the Node to the Host located at the specified address
    #[tracing::instrument(skip_all)]
    pub async fn activate(mut self) -> Result<Node<Nonblocking, Tcp, Active, T>, Error> {
        let addr = self.cfg.network_cfg.host_addr;
        let topic = self.topic.clone();

        let stream: Result<TcpStream, Error> = {
            let stream = try_connection(addr).await?;
            let stream = handshake(stream, topic).await?;
            Ok(stream)
        };
        if let Ok(stream) = stream {
            debug!(
                "Established Node<=>Host TCP stream: {:?}",
                stream.local_addr()
            );
            self.stream = Some(stream);
        }

        Ok(Node::<Nonblocking, Tcp, Active, T>::from(self))
    }

    #[tracing::instrument]
    pub async fn subscribe(
        mut self,
        rate: Duration,
    ) -> Result<Node<Nonblocking, Tcp, Subscription, T>, Error> {
        let addr = self.cfg.network_cfg.host_addr;
        let topic = self.topic.clone();

        let subscription_data: Arc<TokioMutex<Option<Msg<T>>>> = Arc::new(TokioMutex::new(None));
        let data = Arc::clone(&subscription_data);

        let buffer = self.buffer.clone();
        let packet = GenericMsg::subscribe(&topic, rate)?;

        let task_subscribe = tokio::spawn(async move {
            if let Ok(stream) = try_connection(addr).await {
                if let Ok(stream) = handshake(stream, topic.clone()).await {
                    loop {
                        if let Err(e) = run_subscription::<T>(
                            packet.clone(),
                            buffer.clone(),
                            &stream,
                            data.clone(),
                        )
                        .await
                        {
                            error!("{:?}", e);
                        }
                    }
                }
            }
        });
        self.task_subscribe = Some(task_subscribe);

        let mut subscription_node = Node::<Nonblocking, Tcp, Subscription, T>::from(self);
        subscription_node.subscription_data = subscription_data;

        Ok(subscription_node)
    }
}

use crate::node::tcp::{await_response, send_msg};
async fn run_subscription<T: Message>(
    packet: GenericMsg,
    buffer: Arc<TokioMutex<Vec<u8>>>,
    stream: &TcpStream,
    data: Arc<TokioMutex<Option<Msg<T>>>>,
) -> Result<(), Error> {
    send_msg(stream, packet.as_bytes()?).await?;

    let mut buffer = buffer.lock().await;
    loop {
        match await_response(stream, &mut buffer).await {
            Ok(msg) => {
                match TryInto::<Msg<T>>::try_into(msg) {
                    Ok(msg) => {
                        let mut data = data.lock().await;
                        use std::ops::DerefMut;
                        match data.deref_mut() {
                            Some(existing) => {
                                let delta = msg.timestamp - existing.timestamp;
                                // println!("The time difference between msg tx/rx is: {} us",delta);
                                if delta <= chrono::Duration::zero() {
                                    // println!("Data is not newer, skipping to next subscription iteration");
                                    continue;
                                }

                                *data = Some(msg);
                            }
                            None => {
                                *data = Some(msg);
                            }
                        }
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            Err(e) => {
                error!("Subscription Error: {:?}", e);
                continue;
            }
        };
    }
}

//------

impl<T: Message> From<Node<Blocking, Tcp, Idle, T>> for Node<Blocking, Tcp, Active, T> {
    fn from(node: Node<Blocking, Tcp, Idle, T>) -> Self {
        Self {
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
            rt_handle: node.rt_handle,
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

impl<T: Message> From<Node<Blocking, Tcp, Idle, T>> for Node<Blocking, Tcp, Subscription, T> {
    fn from(node: Node<Blocking, Tcp, Idle, T>) -> Self {
        Self {
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
            rt_handle: node.rt_handle,
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

use crate::node::network_config::Blocking;
impl<T: Message + 'static> Node<Blocking, Tcp, Idle, T> {
    /// Attempt connection from the Node to the Host located at the specified address
    #[tracing::instrument(skip_all)]
    pub fn activate(mut self) -> Result<Node<Blocking, Tcp, Active, T>, Error> {
        let addr = self.cfg.network_cfg.host_addr;
        let topic = self.topic.clone();

        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };

        let stream: Result<TcpStream, Error> = handle.block_on(async move {
            let stream = try_connection(addr).await?;
            let stream = handshake(stream, topic).await?;
            Ok(stream)
        });
        if let Ok(stream) = stream {
            debug!(
                "Established Node<=>Host TCP stream: {:?}",
                stream.local_addr()
            );
            self.stream = Some(stream);
        }

        Ok(Node::<Blocking, Tcp, Active, T>::from(self))
    }

    #[tracing::instrument]
    pub fn subscribe(
        mut self,
        rate: Duration,
    ) -> Result<Node<Blocking, Tcp, Subscription, T>, Error> {
        let addr = self.cfg.network_cfg.host_addr;
        let topic = self.topic.clone();

        let subscription_data: Arc<TokioMutex<Option<Msg<T>>>> = Arc::new(TokioMutex::new(None));
        let data = Arc::clone(&subscription_data);

        let buffer = self.buffer.clone();
        let packet = GenericMsg::subscribe(&topic, rate)?;

        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };

        let task_subscribe = handle.spawn(async move {
            if let Ok(stream) = try_connection(addr).await {
                if let Ok(stream) = handshake(stream, topic.clone()).await {
                    loop {
                        if let Err(e) = run_subscription::<T>(
                            packet.clone(),
                            buffer.clone(),
                            &stream,
                            data.clone(),
                        )
                        .await
                        {
                            error!("{:?}", e);
                        }
                    }
                }
            }
        });
        self.task_subscribe = Some(task_subscribe);

        let mut subscription_node = Node::<Blocking, Tcp, Subscription, T>::from(self);
        subscription_node.subscription_data = subscription_data;

        Ok(subscription_node)
    }
}
