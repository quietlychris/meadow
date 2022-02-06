use serde::{Deserialize, Serialize};

/// Msg definitions for publish or request of topic data
#[derive(Debug, Serialize, Deserialize)]
pub enum MsgType {
    SET,
    GET,
}

/// Message format containing a strongly-typed data payload and associated metadata
#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Msg<T> {
    pub msg_type: MsgType,
    pub name: String,
    pub topic: String,
    pub data_type: String,
    pub data: T,
}

/// Message format containing a generic Vec<u8> data payload and associated metadata
#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct GenericMsg {
    pub msg_type: MsgType,
    pub timestamp: String,
    pub name: String,
    pub topic: String,
    pub data_type: String,
    pub data: Vec<u8>,
}

/// Request passed between Node and Host for the desired topic information
#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub topic: String,
    pub ip: String,
    pub type_info: String,
}

/// Example test struct for docs and tests
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
#[repr(C)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
}

/// Example test struct for docs and tests, incompatible with Pose
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct NotPose {
    a: isize,
}
