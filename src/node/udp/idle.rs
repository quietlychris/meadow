extern crate alloc;
use crate::Error;
use crate::*;

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
use crate::node::network_config::Udp;

impl<T: Message + 'static> Node<Udp, Idle, T> {
    //#[tracing::instrument(skip_all)]
    pub fn activate(mut self) -> Result<Node<Udp, Active, T>, Error> {
        match self.runtime.block_on(async move {
            match UdpSocket::bind("[::]:0").await {
                Ok(socket) => Ok(socket),
                Err(_e) => Err(Error::AccessSocket),
            }
        }) {
            Ok(socket) => self.socket = Some(socket),
            Err(e) => return Err(e),
        };

        Ok(Node::<Udp, Active, T>::from(self))
    }

    pub fn subscribe(mut self, rate: Duration) -> Result<Node<Udp, Subscription, T>, Error> {
        let topic = self.topic.clone();
        let subscription_data: Arc<TokioMutex<Option<Msg<T>>>> = Arc::new(TokioMutex::new(None));
        let data = Arc::clone(&subscription_data);
        let addr = self.cfg.network_cfg.host_addr;

        let task_subscribe = self.runtime.spawn(async move {
            dbg!("in task_subscribe()");
            let packet = GenericMsg {
                msg_type: MsgType::GET,
                timestamp: Utc::now(),
                topic: topic.clone(),
                data_type: std::any::type_name::<T>().to_string(),
                data: Vec::new(),
            };
            let socket = UdpSocket::bind("[::]:0").await.unwrap();

            for i in 0..10000 {
                //let mut buffer = buffer.lock().await;

                let mut buffer = vec![0u8; 10_000];
                info!("LOOPED {}", i);
                let packet_as_bytes = to_allocvec(&packet).unwrap();

                info!("about to send msg #{}", i);
                match udp::send_msg(&socket, packet_as_bytes.clone(), addr).await {
                    Ok(n) => {
                        info!(n);
                    }
                    Err(e) => {
                        let e = e.to_string();
                        info!(e);
                    }
                };
                info!("sent msg #{}", i);
                let msg = match udp::await_response::<T>(&socket, &mut buffer).await {
                    Ok(msg) => msg,
                    Err(e) => {
                        panic!("Subscription Error: {}", e);
                        // continue;
                    }
                };
                info!("received reply #{}: {:?}", i, msg);
                let delta = Utc::now() - msg.timestamp;
                // println!("The time difference between msg tx/rx is: {} us",delta);
                if delta <= chrono::Duration::zero() {
                    info!("Data is not newer, skipping to next subscription iteration");
                    // continue;
                }

                let mut data = data.lock().await;

                *data = Some(msg);
                sleep(rate).await;
            }
        });

        self.task_subscribe = Some(task_subscribe);

        let mut subscription_node = Node::<Udp, Subscription, T>::from(self);
        subscription_node.subscription_data = subscription_data;

        Ok(subscription_node)
    }
}

impl<T: Message> From<Node<Udp, Idle, T>> for Node<Udp, Subscription, T> {
    fn from(node: Node<Udp, Idle, T>) -> Self {
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
