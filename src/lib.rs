// If an async Future goes unused, toss a compile-time error
#![deny(unused_must_use)]

pub mod host;
pub mod msg;
pub mod networks;
pub mod node;

// Require that the README examples are valid
// Will fail `cargo test` if not
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadMeDocTests;

pub use crate::host::*;
pub use crate::msg::*;
pub use crate::networks::*;
pub use crate::node::*;
