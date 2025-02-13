extern crate alloc;
use crate::node::network_config::{Nonblocking, Udp};
use crate::Error;

use crate::node::udp::send_msg;
use crate::node::*;

use tokio::net::UdpSocket;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

use tracing::*;

use std::convert::TryInto;
use std::net::SocketAddr;
use std::result::Result;
use std::sync::Arc;

use alloc::vec::Vec;
use postcard::*;
use std::marker::PhantomData;

use crate::msg::*;

impl<T: Message> From<Node<Nonblocking, Udp, Idle, T>> for Node<Nonblocking, Udp, Subscription, T> {
    fn from(node: Node<Nonblocking, Udp, Idle, T>) -> Self {
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

impl<T: Message + 'static> Node<Nonblocking, Udp, Idle, T> {
    #[tracing::instrument(skip(self))]
    pub async fn activate(mut self) -> Result<Node<Nonblocking, Udp, Active, T>, Error> {
        match {
            match UdpSocket::bind("[::]:0").await {
                Ok(socket) => {
                    info!("Bound to socket: {:?}", &socket);
                    Ok(socket)
                }
                Err(_e) => Err(Error::AccessSocket),
            }
        } {
            Ok(socket) => self.socket = Some(socket),
            Err(e) => return Err(e),
        };

        Ok(Node::<Nonblocking, Udp, Active, T>::from(self))
    }

    #[tracing::instrument(skip(self))]
    pub async fn subscribe(
        mut self,
        rate: Duration,
    ) -> Result<Node<Nonblocking, Udp, Subscription, T>, Error> {
        let topic = self.topic.clone();
        let subscription_data: Arc<TokioMutex<Option<Msg<T>>>> = Arc::new(TokioMutex::new(None));
        let data = Arc::clone(&subscription_data);
        let addr = self.cfg.network_cfg.host_addr;
        let buffer = self.buffer.clone();

        let packet = GenericMsg::subscribe(topic, rate)?;

        let task_subscribe = tokio::spawn(async move {
            if let Ok(socket) = UdpSocket::bind("[::]:0").await {
                info!("Bound to socket: {:?}", &socket);
                loop {
                    if let Err(e) = run_subscription::<T>(
                        packet.clone(),
                        buffer.clone(),
                        &socket,
                        data.clone(),
                        addr,
                    )
                    .await
                    {
                        // dbg!(&e);
                        error!("{:?}", e);
                    }
                }
            }
        });

        self.task_subscribe = Some(task_subscribe);

        let mut subscription_node = Node::<Nonblocking, Udp, Subscription, T>::from(self);
        subscription_node.subscription_data = subscription_data;

        Ok(subscription_node)
    }
}

#[tracing::instrument(skip_all)]
async fn run_subscription<T: Message>(
    packet: GenericMsg,
    buffer: Arc<TokioMutex<Vec<u8>>>,
    socket: &UdpSocket,
    data: Arc<TokioMutex<Option<Msg<T>>>>,
    addr: SocketAddr,
) -> Result<(), Error> {
    udp::send_msg(socket, packet.as_bytes()?, addr).await?;

    loop {
        let msg: Msg<T> = udp::await_response(socket, buffer.clone())
            .await?
            .try_into()?;
        info!("UDP Msg<T> received: {:?}", &msg);
        let delta = Utc::now() - msg.timestamp;
        if delta <= chrono::Duration::zero() {
            info!("Data is not newer, skipping to next subscription iteration");
            continue;
        }

        let mut data = data.lock().await;
        *data = Some(msg);
        info!("Inserted new subscription data!");
    }
}

//--------

use crate::node::network_config::Blocking;

impl<T: Message> From<Node<Blocking, Udp, Idle, T>> for Node<Blocking, Udp, Subscription, T> {
    fn from(node: Node<Blocking, Udp, Idle, T>) -> Self {
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

impl<T: Message + 'static> Node<Blocking, Udp, Idle, T> {
    #[tracing::instrument(skip(self))]
    pub fn activate(mut self) -> Result<Node<Blocking, Udp, Active, T>, Error> {
        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };

        match handle.block_on(async move {
            match UdpSocket::bind("[::]:0").await {
                Ok(socket) => {
                    info!("Bound to socket: {:?}", &socket);
                    Ok(socket)
                }
                Err(_e) => Err(Error::AccessSocket),
            }
        }) {
            Ok(socket) => self.socket = Some(socket),
            Err(e) => return Err(e),
        };

        Ok(Node::<Blocking, Udp, Active, T>::from(self))
    }

    #[tracing::instrument(skip(self))]
    pub fn subscribe(
        mut self,
        rate: Duration,
    ) -> Result<Node<Blocking, Udp, Subscription, T>, Error> {
        let topic = self.topic.clone();
        let subscription_data: Arc<TokioMutex<Option<Msg<T>>>> = Arc::new(TokioMutex::new(None));
        let data = Arc::clone(&subscription_data);
        let addr = self.cfg.network_cfg.host_addr;
        let buffer = self.buffer.clone();

        let packet = GenericMsg::subscribe(topic, rate)?;

        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };

        let task_subscribe = handle.spawn(async move {
            if let Ok(socket) = UdpSocket::bind("[::]:0").await {
                info!("Bound to socket: {:?}", &socket);
                loop {
                    if let Err(e) = run_subscription::<T>(
                        packet.clone(),
                        buffer.clone(),
                        &socket,
                        data.clone(),
                        addr,
                    )
                    .await
                    {
                        // dbg!(&e);
                        error!("{:?}", e);
                    }
                }
            }
        });

        self.task_subscribe = Some(task_subscribe);

        let mut subscription_node = Node::<Blocking, Udp, Subscription, T>::from(self);
        subscription_node.subscription_data = subscription_data;

        Ok(subscription_node)
    }
}
