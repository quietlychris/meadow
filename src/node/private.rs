
pub trait Sealed {}
impl Sealed for crate::Udp {}
impl Sealed for crate::Tcp {}
#[cfg(feature = "quic")]
impl Sealed for crate::node::network_config::Quic {}
// impl Sealed for crate::Quic {}

impl Sealed for crate::Idle {}
impl Sealed for crate::Active {}
