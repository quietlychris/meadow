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

impl<T: Message> From<Node<Tcp, Idle, T>> for Node<Tcp, Subscription, T> {
    fn from(node: Node<Tcp, Idle, T>) -> Self {
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

impl<T: Message + 'static> Node<Tcp, Idle, T> {
    /// Attempt connection from the Node to the Host located at the specified address
    #[tracing::instrument(skip_all)]
    pub fn activate(mut self) -> Result<Node<Tcp, Active, T>, Error> {
        let addr = self.cfg.network_cfg.host_addr;
        let topic = self.topic.clone();

        let stream: Result<TcpStream, Error> = self.rt_handle.block_on(async move {
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

        Ok(Node::<Tcp, Active, T>::from(self))
    }

    #[tracing::instrument]
    pub fn subscribe(mut self, rate: Duration) -> Result<Node<Tcp, Subscription, T>, Error> {
        let addr = self.cfg.network_cfg.host_addr;
        let topic = self.topic.clone();

        let subscription_data: Arc<TokioMutex<Option<Msg<T>>>> = Arc::new(TokioMutex::new(None));
        let data = Arc::clone(&subscription_data);

        let buffer = self.buffer.clone();
        let packet = GenericMsg {
            msg_type: MsgType::SUBSCRIBE,
            timestamp: Utc::now(),
            topic: topic.clone(),
            data_type: std::any::type_name::<T>().to_string(),
            data: to_allocvec(&rate)?,
        };

        let task_subscribe = self.rt_handle.spawn(async move {
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

        let mut subscription_node = Node::<Tcp, Subscription, T>::from(self);
        subscription_node.subscription_data = subscription_data;

        Ok(subscription_node)
    }
}

async fn run_subscription<T: Message>(
    packet: GenericMsg,
    buffer: Arc<TokioMutex<Vec<u8>>>,
    stream: &TcpStream,
    data: Arc<TokioMutex<Option<Msg<T>>>>,
) -> Result<(), Error> {
    let packet_as_bytes = to_allocvec(&packet)?;
    send_msg(stream, packet_as_bytes).await?;

    let mut buffer = buffer.lock().await;
    loop {
        match await_response::<T>(stream, &mut buffer).await {
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
                error!("Subscription Error: {:?}", e);
                continue;
            }
        };
    }
}
