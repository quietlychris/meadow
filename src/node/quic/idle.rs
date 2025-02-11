extern crate alloc;
use crate::error::{Error, Quic::*};
use crate::*;

use crate::node::network_config::{Nonblocking, Quic};
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
use std::convert::TryInto;
use std::marker::PhantomData;

// Quic
use quinn::Endpoint;

use crate::msg::*;
use crate::node::quic::generate_client_config_from_certs;
use chrono::Utc;

impl<T: Message> From<Node<Nonblocking, Quic, Idle, T>> for Node<Nonblocking, Quic, Active, T> {
    fn from(node: Node<Nonblocking, Quic, Idle, T>) -> Self {
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
            endpoint: node.endpoint,
            connection: node.connection,
            subscription_data: node.subscription_data,
            task_subscribe: None,
        }
    }
}

impl<T: Message> From<Node<Nonblocking, Quic, Idle, T>>
    for Node<Nonblocking, Quic, Subscription, T>
{
    fn from(node: Node<Nonblocking, Quic, Idle, T>) -> Self {
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
            endpoint: node.endpoint,
            connection: node.connection,
            subscription_data: node.subscription_data,
            task_subscribe: node.task_subscribe,
        }
    }
}

impl<T: Message + 'static> Node<Nonblocking, Quic, Idle, T> {
    /// Attempt connection from the Node to the Host located at the specified address
    //#[tracing::instrument(skip_all)]
    pub async fn activate(mut self) -> Result<Node<Nonblocking, Quic, Active, T>, Error> {
        debug!("Attempting QUIC connection");

        self.create_connection().await?;

        Ok(Node::<Nonblocking, Quic, Active, T>::from(self))
    }

    async fn create_connection(&mut self) -> Result<(), Error> {
        let host_addr = self.cfg.network_cfg.host_addr;
        let cert_path = self.cfg.network_cfg.cert_path.clone();

        let (endpoint, connection) = {
            // QUIC, needs to be done inside of a tokio context
            let client_cfg = generate_client_config_from_certs(cert_path)?;
            let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

            let mut endpoint = Endpoint::client(client_addr)?;
            endpoint.set_default_client_config(client_cfg);

            // TO_DO: This shouldn't just be "localhost"
            let connection = endpoint
                .connect(host_addr, "localhost")
                .map_err(|e| Into::<error::Quic>::into(e))?
                // .map_err(ConnectError)?
                .await
                .map_err(|e| Into::<error::Quic>::into(e))?;
            // .map_err(ConnectionError)?;

            debug!("{:?}", &endpoint.local_addr());

            Ok::<(Endpoint, quinn::Connection), Error>((endpoint, connection))
        }?;
        self.endpoint = Some(endpoint);
        self.connection = Some(connection);
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn subscribe(
        mut self,
        rate: Duration,
    ) -> Result<Node<Nonblocking, Quic, Subscription, T>, Error> {
        self.create_connection().await?;
        let connection = self.connection.clone();
        let topic = self.topic.clone();

        let subscription_data: Arc<TokioMutex<Option<Msg<T>>>> = Arc::new(TokioMutex::new(None));
        let data = Arc::clone(&subscription_data);

        let buffer = self.buffer.clone();

        let packet = GenericMsg {
            msg_type: MsgType::Subscribe,
            timestamp: Utc::now(),
            topic: topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: postcard::to_allocvec(&rate)?,
        };

        let task_subscribe = tokio::spawn(async move {
            if let Some(connection) = connection {
                loop {
                    if let Err(e) = run_subscription::<T>(
                        packet.clone(),
                        buffer.clone(),
                        connection.clone(),
                        data.clone(),
                    )
                    .await
                    {
                        error!("{:?}", e);
                    }
                }
            }
        });

        self.task_subscribe = Some(task_subscribe);

        let mut subscription_node = Node::<Nonblocking, Quic, Subscription, T>::from(self);
        subscription_node.subscription_data = subscription_data;

        Ok(subscription_node)
    }
}

#[tracing::instrument(skip_all)]
async fn run_subscription<T: Message>(
    packet: GenericMsg,
    buffer: Arc<TokioMutex<Vec<u8>>>,
    connection: quinn::Connection,
    data: Arc<TokioMutex<Option<Msg<T>>>>,
) -> Result<(), Error> {
    let packet_as_bytes: Vec<u8> = to_allocvec(&packet)?;
    let (mut send, mut recv) = connection.open_bi().await?;

    send.write_all(&packet_as_bytes).await?;
    send.finish().await?;

    loop {
        let mut buf = buffer.lock().await;

        if let Ok(Some(n)) = recv.read(&mut buf).await {
            let bytes = &buf[..n];

            let generic = from_bytes::<GenericMsg>(bytes)?;
            info!("QUIC received generic: {:?}", &generic);
            let msg: Msg<T> = generic.try_into()?;

            if let Some(data) = data.lock().await.as_ref() {
                debug!("Timestamp: {}", data.timestamp);
                let delta = msg.timestamp - data.timestamp;
                info!(
                    "The time difference between QUIC subscription msg tx/rx is: {} us",
                    delta
                );
                if delta <= chrono::Duration::zero() {
                    warn!("Data is not newer, skipping to next subscription iteration");
                    continue;
                }
            }

            let mut data = data.lock().await;
            *data = Some(msg);
        }
    }
}

// -----------------

use crate::node::network_config::Blocking;

impl<T: Message> From<Node<Blocking, Quic, Idle, T>> for Node<Blocking, Quic, Active, T> {
    fn from(node: Node<Blocking, Quic, Idle, T>) -> Self {
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
            endpoint: node.endpoint,
            connection: node.connection,
            subscription_data: node.subscription_data,
            task_subscribe: None,
        }
    }
}

impl<T: Message> From<Node<Blocking, Quic, Idle, T>> for Node<Blocking, Quic, Subscription, T> {
    fn from(node: Node<Blocking, Quic, Idle, T>) -> Self {
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
            endpoint: node.endpoint,
            connection: node.connection,
            subscription_data: node.subscription_data,
            task_subscribe: node.task_subscribe,
        }
    }
}

impl<T: Message + 'static> Node<Blocking, Quic, Idle, T> {
    /// Attempt connection from the Node to the Host located at the specified address
    //#[tracing::instrument(skip_all)]
    pub fn activate(mut self) -> Result<Node<Blocking, Quic, Active, T>, Error> {
        debug!("Attempting QUIC connection");

        self.create_connection()?;

        Ok(Node::<Blocking, Quic, Active, T>::from(self))
    }

    fn create_connection(&mut self) -> Result<(), Error> {
        let host_addr = self.cfg.network_cfg.host_addr;
        let cert_path = self.cfg.network_cfg.cert_path.clone();

        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };

        let (endpoint, connection) = handle.block_on(async move {
            // QUIC, needs to be done inside of a tokio context
            let client_cfg = generate_client_config_from_certs(cert_path)?;
            let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

            let mut endpoint = Endpoint::client(client_addr)?;
            endpoint.set_default_client_config(client_cfg);

            // TO_DO: This shouldn't just be "localhost"
            let connection = endpoint.connect(host_addr, "localhost")?.await?;

            debug!("{:?}", &endpoint.local_addr());

            Ok::<(Endpoint, quinn::Connection), Error>((endpoint, connection))
        })?;
        self.endpoint = Some(endpoint);
        self.connection = Some(connection);
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub fn subscribe(
        mut self,
        rate: Duration,
    ) -> Result<Node<Blocking, Quic, Subscription, T>, Error> {
        self.create_connection()?;
        let connection = self.connection.clone();
        let topic = self.topic.clone();

        let subscription_data: Arc<TokioMutex<Option<Msg<T>>>> = Arc::new(TokioMutex::new(None));
        let data = Arc::clone(&subscription_data);

        let buffer = self.buffer.clone();

        let packet = GenericMsg {
            msg_type: MsgType::Subscribe,
            timestamp: Utc::now(),
            topic: topic.to_string(),
            data_type: std::any::type_name::<T>().to_string(),
            data: postcard::to_allocvec(&rate)?,
        };

        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };
        let task_subscribe = handle.spawn(async move {
            if let Some(connection) = connection {
                loop {
                    if let Err(e) = run_subscription::<T>(
                        packet.clone(),
                        buffer.clone(),
                        connection.clone(),
                        data.clone(),
                    )
                    .await
                    {
                        error!("{:?}", e);
                    }
                }
            }
        });

        self.task_subscribe = Some(task_subscribe);

        let mut subscription_node = Node::<Blocking, Quic, Subscription, T>::from(self);
        subscription_node.subscription_data = subscription_data;

        Ok(subscription_node)
    }
}
