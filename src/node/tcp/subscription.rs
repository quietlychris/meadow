use crate::node::network_config::{Blocking, Nonblocking, Tcp};
use crate::node::{Node, Subscription};
use crate::{Error, Message, Msg};
use std::ops::Deref;

impl<T: Message + 'static> Node<Nonblocking, Tcp, Subscription, T> {
    pub async fn get_subscribed_data(&self) -> Result<Msg<T>, crate::Error> {
        let data = self.subscription_data.lock().await.clone();
        if let Some(msg) = data {
            Ok(msg)
        } else {
            Err(Error::NoSubscriptionValue)
        }
    }
}

//----

impl<T: Message + 'static> Node<Blocking, Tcp, Subscription, T> {
    pub fn get_subscribed_data(&self) -> Result<Msg<T>, crate::Error> {
        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };
        handle.block_on(async {
            let data = self.subscription_data.lock().await.clone();
            if let Some(msg) = data {
                Ok(msg)
            } else {
                Err(Error::NoSubscriptionValue)
            }
        })
    }
}
