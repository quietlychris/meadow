#![deny(missing_docs)]

//! `bissel` is an experimental robotics-focused publish/request
//! middleware for embedded Linux. It uses a star-shaped network topology,
//! with a focus on ease-of-use and transparent design and operation.
//! It is more similar to [ZeroMQ](https://zguide.zeromq.org/docs/chapter1/) than to higher-level frameworks
//! like [ROS/2](https://design.ros2.org/articles/discovery_and_negotiation.html), but uses central coordination process similar to [MOOS-IvP](https://oceanai.mit.edu/ivpman/pmwiki/pmwiki.php?n=Helm.HelmDesignIntro#section2.4).

pub use bissel_internal::*;

#[cfg(feature = "dynamic")]
#[allow(unused_imports)]
use bissel_dylib::*;
