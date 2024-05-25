use crate::error::Error;
use crate::msg::Message;
use crate::node::nonblocking::network_config::Quic;
use crate::node::nonblocking::Subscription;
use crate::Msg;
use crate::Node;

impl<T: Message + 'static> Node<Quic, Subscription, T> {
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
