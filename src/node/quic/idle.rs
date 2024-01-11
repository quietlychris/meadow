extern crate alloc;
use crate::error::{Error, Quic::*};
use crate::*;

use crate::node::network_config::Quic;
use crate::node::*;

use std::path::PathBuf;

use tokio::net::UdpSocket;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

use tracing::*;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::result::Result;
use std::sync::Arc;

use alloc::vec::Vec;
use postcard::*;
use std::marker::PhantomData;

// Quic
use quinn::Endpoint;

use crate::msg::*;
use crate::node::quic::generate_client_config_from_certs;
use chrono::Utc;

impl<T: Message> From<Node<Quic, Idle, T>> for Node<Quic, Active, T> {
    fn from(node: Node<Quic, Idle, T>) -> Self {
        Self {
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
            stream: node.stream,
            topic: node.topic,
            socket: node.socket,
            buffer: node.buffer,
            endpoint: node.endpoint,
            connection: node.connection,
            subscription_data: node.subscription_data,
            task_subscribe: None,
        }
    }
}

impl<T: Message> From<Node<Quic, Idle, T>> for Node<Quic, Subscription, T> {
    fn from(node: Node<Quic, Idle, T>) -> Self {
        Self {
            __state: PhantomData,
            __data_type: PhantomData,
            cfg: node.cfg,
            runtime: node.runtime,
            stream: node.stream,
            topic: node.topic,
            socket: node.socket,
            buffer: node.buffer,
            endpoint: node.endpoint,
            connection: node.connection,
            subscription_data: node.subscription_data,
            task_subscribe: node.task_subscribe,
        }
    }
}

impl<T: Message + 'static> Node<Quic, Idle, T> {
    /// Attempt connection from the Node to the Host located at the specified address
    //#[tracing::instrument(skip_all)]
    pub fn activate(mut self) -> Result<Node<Quic, Active, T>, Error> {
        debug!("Attempting QUIC connection");

        self.create_connection()?;

        Ok(Node::<Quic, Active, T>::from(self))
    }

    fn create_connection(&mut self) -> Result<(), Error> {
        let host_addr = self.cfg.network_cfg.host_addr;
        let cert_path = self.cfg.network_cfg.cert_path.clone();

        let (endpoint, connection) = self.runtime.block_on(async move {
            // QUIC, needs to be done inside of a tokio context
            let client_cfg = generate_client_config_from_certs(cert_path)?;
            let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

            let mut endpoint = Endpoint::client(client_addr)?;
            endpoint.set_default_client_config(client_cfg);

            // TO_DO: This shouldn't just be "localhost"
            let connection = endpoint
                .connect(host_addr, "localhost")
                .map_err(QuinnConnect)?
                .await
                .map_err(QuinnConnection)?;

            debug!("{:?}", &endpoint.local_addr());

            Ok::<(Endpoint, quinn::Connection), Error>((endpoint, connection))
        })?;
        self.endpoint = Some(endpoint);
        self.connection = Some(connection);
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub fn subscribe(mut self, rate: Duration) -> Result<Node<Quic, Subscription, T>, Error> {
        self.create_connection()?;
        let connection = self.connection.clone();
        let topic = self.topic.clone();

        let subscription_data: Arc<TokioMutex<Option<Msg<T>>>> = Arc::new(TokioMutex::new(None));
        let data = Arc::clone(&subscription_data);

        let buffer = self.buffer.clone();
        let task_subscribe = self.runtime.spawn(async move {
            let packet = GenericMsg {
                msg_type: MsgType::GET,
                timestamp: Utc::now(),
                topic: topic.to_string(),
                data_type: std::any::type_name::<T>().to_string(),
                data: Vec::new(),
            };

            let mut buf = buffer.lock().await;

            loop {
                let packet_as_bytes: Vec<u8> = match to_allocvec(&packet) {
                    Ok(packet) => packet,
                    _ => continue,
                };

                if let Some(connection) = connection.clone() {
                    match connection.open_bi().await {
                        Ok((mut send, mut recv)) => {
                            if let Ok(()) = send.write_all(&packet_as_bytes).await {
                                if let Ok(()) = send.finish().await {
                                    debug!("Node successfully wrote packet to stream");
                                }
                            } else {
                                error!("Error writing packet to stream");
                            }

                            match recv.read(&mut buf).await {
                                //Ok(0) => Err(Error::QuicIssue),
                                Ok(Some(n)) => {
                                    let bytes = &buf[..n];

                                    match from_bytes::<Msg<T>>(bytes) {
                                        Ok(msg) => {
                                            {

                                                if let Some(data) = data.lock().await.as_ref() {
                                                    debug!("Timestamp: {}", data.timestamp);
                                                    let delta =
                                                        data.timestamp - msg.timestamp;
                                                    debug!("The time difference between QUIC subscription msg tx/rx is: {} us",delta);
                                                    if delta <= chrono::Duration::zero() {
                                                        // println!("Data is not newer, skipping to next subscription iteration");
                                                        continue;
                                                    }
                                                }
                                            }

                                            debug!("QUIC Subscriber received new data");
                                            let mut data = data.lock().await;
                                            *data = Some(msg);
                                            sleep(rate).await;

                                        }
                                        Err(_) => continue,
                                    }
                                }
                                _ => {
                                    // // if e.kind() == std::io::ErrorKind::WouldBlock {}
                                    continue;
                                }
                            }
                        }
                        _ => continue,
                    };
                }
            }
        });
        self.task_subscribe = Some(task_subscribe);

        let mut subscription_node = Node::<Quic, Subscription, T>::from(self);
        subscription_node.subscription_data = subscription_data;

        Ok(subscription_node)
    }
}
