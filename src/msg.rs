use serde::{Deserialize, Serialize};
// use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize)]
pub struct RhizaMsg<T> {
    pub name: String,
    pub type_info: String,
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericRhizaMsg {
    pub name: String,
    pub type_info: String,
    pub data: Vec<u8>,
}

// TO_DO: IpAddr can't be serialized, but we can do a parse check
// on the node side s.t. we're sure it will be okay
// for the host to respond to (check on the host side as well)
#[derive(Debug, Serialize, Deserialize)]
pub struct RhizaRequest {
    pub name: String,
    pub ip: String,
    pub type_info: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
}
