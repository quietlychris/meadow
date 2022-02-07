use crate::*;

use std::error::Error;
use std::result::Result;

impl<T: Message + 'static> Node<Subscription, T> {
    // Should actually return a <T>
    pub fn get_subscribed_data(&self) -> Result<Option<T>, Box<dyn Error>> {
        let data = self.subscription_data.clone();
        let result = self.runtime.block_on(async {
            let data = data.lock().await;
            match data.clone() {
                Some(value) => Some(value.data),
                None => None,
            }
        });
        Ok(result)
    }
}
