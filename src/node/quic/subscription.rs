// use crate::msg::Message;
// use crate::Error;
// use crate::Quic;
use crate::error::Error;
use crate::msg::Message;
use crate::node::network_config::Quic;
use crate::node::Subscription;
use crate::Node;

impl<T: Message + 'static> Node<Quic, Subscription, T> {
    // Should actually return a <T>
    pub fn get_subscribed_data(&self) -> Result<T, Error> {
        let data = self.subscription_data.clone();
        self.rt_handle.block_on(async {
            let data = data.lock().await;
            match data.clone() {
                Some(value) => Ok(value.data),
                None => Err(Error::NoSubscriptionValue),
            }
        })
    }
}
