#[cfg(feature = "quic")]
use meadow::host::quic::generate_certs;

fn main() {
    #[cfg(feature = "quic")]
    generate_certs().unwrap();
    #[cfg(not(feature = "quic"))]
    panic!("Must enable the \"quic\" feature to run");
}
