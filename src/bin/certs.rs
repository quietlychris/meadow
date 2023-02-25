fn main() {
    #[cfg(feature = "quic")]
    meadow::generate_certs().unwrap();
    #[cfg(not(features = "quic"))]
    panic!("Must enable the \"quic\" feature to run");
}
