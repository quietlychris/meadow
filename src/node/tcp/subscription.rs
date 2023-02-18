use crate::*;

// use std::result::Result;

impl<T: Message + 'static> Node<Tcp, Subscription, T> {
    // Should actually return a <T>
    pub fn get_subscribed_data(&self) -> Result<T, crate::Error> {
        let data = self.subscription_data.clone();
        self.runtime.block_on(async {
            let data = data.lock().await;
            match data.clone() {
                Some(value) => Ok(value.data),
                None => Err(Error::NoSubscriptionValue),
            }
        })
    }
}
