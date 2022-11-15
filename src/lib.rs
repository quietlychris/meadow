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
//! Meadow currently supports the following messaging patterns:
//!
//! | Protocol | Publish   | Request    | Subscribe |
//! |----------|-----------|------------|-----------|
//! | TCP      | **X**     | **X**      | **X**     |
//! | UDP      | **X**     |            |           |
//!

/// Central coordination process, which stores published data and responds to requests
pub mod host;
/// Message definitions for publish/request functions
pub mod msg;
/// Network-based utility module
pub mod networks;
/// Named objects that publish and request strongly-typed data to named topics on the Host
pub mod node;

pub mod error;

// /// Re-export sled for building the key-value store configuration
pub use serde::{Deserialize, Serialize};
#[doc(hidden)]
pub use sled::Config as SledConfig;

// Require that the README examples are valid
// Will fail `cargo test` if not
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadMeDocTests;

pub use crate::host::*;
pub use crate::node::*;

pub use crate::msg::*;
pub use crate::networks::*;

pub use crate::error::Error;
