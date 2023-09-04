// If an async Future goes unused, toss a compile-time error
#![deny(unused_must_use)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![allow(unused_imports)]

//! `meadow` is an experimental robotics-focused publish/request middleware
//! for embedded Linux. It uses a star-shaped network topology, with a focus
//! on ease-of-use and transparent design and operation. It is more similar to
//! [`ZeroMQ`](https://zguide.zeromq.org/docs/chapter1/) than to higher-level frameworks like [ROS/2](https://design.ros2.org/articles/discovery_and_negotiation.html),
//! but uses a central coordination process similar to [MOOS-IvP](https://oceanai.mit.edu/ivpman/pmwiki/pmwiki.php?n=Helm.HelmDesignIntro#section2.4).
//! Meadow currently supports the following messaging patterns over different
//! transport protocols:
//!
//!| Protocol | Publish   | Request    | Subscribe | Encryption |
//!|----------|-----------|------------|-----------|------------|
//!| TCP      | **X**     | **X**      | **X**     |            |
//!| UDP      | **X**     |            |           |            |
//!| QUIC     | **X**     | **X**      | **X**     | **X**      |
//!

/// Error types used by Meadow
pub mod error;
/// Central coordination process, which stores published data and responds to requests
pub mod host;
/// Message definitions for publish/request functions
pub mod msg;
/// Network-based utility module
pub mod networks;
/// Named objects that publish and request strongly-typed data to named topics on the Host
pub mod node;

pub use serde::{Deserialize, Serialize};

#[doc(hidden)]
pub use sled::Config as SledConfig;

// Require that the README examples are valid
// Will fail `cargo test` if not
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadMeDocTests;

pub use crate::error::Error;
pub use crate::host::{Host, HostConfig};
pub use crate::msg::{GenericMsg, Message, Msg, MsgType};
pub use crate::networks::get_ip;
pub use crate::node::{
    tcp::await_response, tcp::send_msg, Active, Idle, NetworkConfig, Node, NodeConfig,
    Subscription, Tcp, Udp,
};
