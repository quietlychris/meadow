#[cfg(feature = "quic")]
use meadow::host::quic::generate_certs;
#[cfg(feature = "quic")]
use meadow::host::quic::QuicCertGenConfig;

fn main() {
    #[cfg(feature = "quic")]
    generate_certs(QuicCertGenConfig::default());
    #[cfg(not(feature = "quic"))]
    panic!("Must enable the \"quic\" feature to run");
}
