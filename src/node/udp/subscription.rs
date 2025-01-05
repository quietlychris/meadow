use crate::node::network_config::{Nonblocking, Udp};
use crate::node::Node;
use crate::node::Subscription;
use crate::{Error, Message, Msg};
use std::ops::Deref;

impl<T: Message + 'static> Node<Nonblocking, Udp, Subscription, T> {
    // Should actually return a <T>
    pub async fn get_subscribed_data(&self) -> Result<Msg<T>, crate::Error> {
        let data = self.subscription_data.lock().await.clone();
        if let Some(msg) = data {
            Ok(msg)
        } else {
            Err(Error::NoSubscriptionValue)
        }
    }
}
