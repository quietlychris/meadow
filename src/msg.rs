use crate::Error;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::convert::{Into, TryInto};

use std::fmt::Debug;
/// Trait for Meadow-compatible data, requiring serde De\Serialize, Debug, and Clone
pub trait Message: Serialize + DeserializeOwned + Debug + Sync + Send + Clone {}
impl<T> Message for T where T: Serialize + DeserializeOwned + Debug + Sync + Send + Clone {}

/// Msg definitions for publish or request of topic data
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub enum MsgType {
    SET,
    GET,
    TOPICS,
    SUBSCRIBE,
}

/// Message format containing a strongly-typed data payload and associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct Msg<T> {
    pub msg_type: MsgType,
    pub timestamp: DateTime<Utc>,
    pub topic: String,
    pub data_type: String,
    pub data: T,
}

/// Message format containing a generic `Vec<u8>` data payload and associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct GenericMsg {
    pub msg_type: MsgType,
    pub timestamp: DateTime<Utc>,
    pub topic: String,
    pub data_type: String,
    pub data: Vec<u8>,
}

impl<T: Message> TryInto<Msg<T>> for GenericMsg {
    type Error = crate::Error;

    fn try_into(self) -> Result<Msg<T>, Error> {
        let data = postcard::from_bytes::<T>(&self.data[..])?;
        Ok(Msg {
            msg_type: self.msg_type,
            timestamp: self.timestamp,
            topic: self.topic.clone(),
            data_type: self.data_type.clone(),
            data,
        })
    }
}
