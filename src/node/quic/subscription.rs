use crate::error::Error;
use crate::msg::Message;
use crate::node::network_config::{Nonblocking, Quic};
use crate::node::Node;
use crate::node::Subscription;
use crate::Msg;

impl<T: Message + 'static> Node<Nonblocking, Quic, Subscription, T> {
    // Should actually return a <T>
    pub async fn get_subscribed_data(&self) -> Result<Msg<T>, Error> {
        let data = self.subscription_data.clone();
        let data = data.lock().await;
        match data.clone() {
            Some(data) => Ok(data),
            None => Err(Error::NoSubscriptionValue),
        }
    }
}

// ----------

use crate::node::network_config::Blocking;

impl<T: Message + 'static> Node<Blocking, Quic, Subscription, T> {
    // Should actually return a <T>
    pub fn get_subscribed_data(&self) -> Result<Msg<T>, Error> {
        let data = self.subscription_data.clone();
        let handle = match &self.rt_handle {
            Some(handle) => handle,
            None => return Err(Error::HandleAccess),
        };
        handle.block_on(async {
            let data = data.lock().await;
            match data.clone() {
                Some(data) => Ok(data),
                None => Err(Error::NoSubscriptionValue),
            }
        })
    }
}
