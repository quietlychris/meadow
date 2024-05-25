use crate::*;
use std::ops::Deref;

impl<T: Message + 'static> Node<Tcp, Subscription, T> {
    pub async fn get_subscribed_data(&self) -> Result<Msg<T>, crate::Error> {
        let data = self.subscription_data.lock().await.clone();
        if let Some(msg) = data {
            Ok(msg)
        } else {
            Err(Error::NoSubscriptionValue)
        }
    }
}
