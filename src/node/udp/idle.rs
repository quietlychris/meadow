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

use crate::msg::*;
use crate::node::network_config::Udp;
use chrono::Utc;

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
}
