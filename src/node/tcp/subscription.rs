use crate::{Msg, Error, Message};
use crate::node::network_config::{Nonblocking, Tcp};
use crate::node::{Node, Subscription};
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
