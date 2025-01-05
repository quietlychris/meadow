extern crate alloc;
use crate::node::network_config::{Nonblocking, Udp};
use crate::Error;

use crate::node::udp::send_msg;
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

        let packet = GenericMsg {
            msg_type: MsgType::SUBSCRIBE,
            timestamp: Utc::now(),
            topic: topic.clone(),
            data_type: std::any::type_name::<T>().to_string(),
            data: to_allocvec(&rate)?,
        };

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
    let packet_as_bytes: Vec<u8> = to_allocvec(&packet)?;
    udp::send_msg(socket, packet_as_bytes.clone(), addr).await?;

    loop {
        let msg = udp::await_response::<T>(socket, buffer.clone()).await?;
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
