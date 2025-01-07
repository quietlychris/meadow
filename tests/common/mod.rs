#![deny(unused_must_use)]

use meadow::prelude::*;

/// Example test struct for docs and tests
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
#[repr(C)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
}

/// Example test struct for docs and tests, incompatible with Pose
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct NotPose {
    a: isize,
}
