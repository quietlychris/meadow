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
    /// Request SET operation on Host
    SET,
    /// Request GET operation on Host
    GET,
    /// Request list of topics from Host  
    TOPICS,
    /// Request start of subscribe operation from Host
    SUBSCRIBE,
}

/// Message format containing a strongly-typed data payload and associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct Msg<T> {
    /// Type of `meadow` message
    pub msg_type: MsgType,
    /// Message timestamp in Utc
    pub timestamp: DateTime<Utc>,
    /// Topic name
    pub topic: String,
    /// Name of message's data type (`String`-typed)
    pub data_type: String,
    /// Strongly-typed data payload
    pub data: T,
}

/// Message format containing a generic `Vec<u8>` data payload and associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct GenericMsg {
    /// Type of `meadow` message
    pub msg_type: MsgType,
    /// Message timestamp in Utc
    pub timestamp: DateTime<Utc>,
    /// Topic name
    pub topic: String,
    /// Name of message's data type (`String`-typed)
    pub data_type: String,
    /// Generic byte-represented data payload
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
