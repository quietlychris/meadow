// If an async Future goes unused, toss a compile-time error
#![deny(unused_must_use)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![allow(unused_imports)]

//! `meadow` is an experimental robotics-focused publish/request middleware
//! for embedded Linux. It is built with a high preference for catching errors
//! at compile-time over runtime and a focus on developer ergonomics, and can
//! natively operate on any [`serde`](https://serde.rs/)-compatible data type.
//!
//! It uses a star-shaped network topology, with a focus
//! on ease-of-use and transparent design and operation. It is more similar to
//! [ZeroMQ](https://zguide.zeromq.org/docs/chapter1/) than to higher-level frameworks like [ROS/2](https://design.ros2.org/articles/discovery_and_negotiation.html),
//! but uses a central coordination process similar to [MOOS-IvP](https://oceanai.mit.edu/ivpman/pmwiki/pmwiki.php?n=Helm.HelmDesignIntro#section2.4).
//! Meadow currently supports the following messaging patterns over different
//! transport protocols:
//!
//!| Protocol | Publish   | Request    | Subscribe | Encryption |
//!|----------|-----------|------------|-----------|------------|
//!| TCP      | **X**     | **X**      | **X**     |            |
//!| UDP      | **X**     | **X**      | **X**     |            |
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
/// Objects that publish and request strongly-typed data to named topics on the Host
pub mod node;

/// Re-export of Serde's `Serialize` and `Deserialize` traits
pub use serde::{Deserialize, Serialize};

// Require that the README examples are valid
// Will fail `cargo test` if not
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadMeDocTests;

pub use crate::error::Error;

pub mod prelude {

    pub use crate::msg::{GenericMsg, Message, Msg, MsgType};
    pub use crate::networks::get_ip;
    pub use crate::*;

    pub use crate::host::{Host, HostConfig, SledConfig, UdpConfig};
    pub use crate::node::config::NodeConfig;
    pub use crate::node::config::RuntimeConfig;
    pub use crate::node::network_config::{Blocking, NetworkConfig, Nonblocking, Tcp, Udp};
    pub use crate::node::{Active, Idle, Node, Subscription};

    #[cfg(feature = "quic")]
    pub use crate::host::{generate_certs, QuicConfig};
    #[cfg(feature = "quic")]
    pub use crate::node::network_config::Quic;
}
