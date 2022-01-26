use serde::{Deserialize, Serialize};
// use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize)]
pub enum Msg {
    SET,
    GET,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct RhizaMsg<T> {
    pub msg_type: Msg,
    pub name: String,
    pub topic: String,
    pub data_type: String,
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct GenericRhizaMsg {
    pub msg_type: Msg,
    pub name: String,
    pub topic: String,
    pub data_type: String,
    pub data: Vec<u8>,
}

// TO_DO: IpAddr can't be serialized, but we can do a parse check
// on the node side s.t. we're sure it will be okay
// for the host to respond to (check on the host side as well)
#[derive(Debug, Serialize, Deserialize)]
pub struct RhizaRequest {
    pub topic: String,
    pub ip: String,
    pub type_info: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
#[repr(C)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct NotPose {a: isize}
