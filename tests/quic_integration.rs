#![cfg(feature = "quic")]
use meadow::*;

mod common;
use common::*;

use std::thread;
use std::time::Duration;

#[cfg(feature = "quic")]
use std::sync::Once;

#[cfg(feature = "quic")]
use meadow::host::quic::generate_certs;

#[cfg(feature = "quic")]
static INIT: Once = Once::new();

#[cfg(feature = "quic")]
pub fn initialize() {
    use meadow::host::quic::QuicCertGenConfig;

    INIT.call_once(|| {
        generate_certs(QuicCertGenConfig::default());
    });
}

#[test]
#[cfg(feature = "quic")]
fn integrate_host_and_single_node_quic() {
    initialize();
    let mut host: Host = HostConfig::default()
        .with_udp_config(None)
        .build()
        .expect("Error building Host with default QUIC configuration");
    host.start().expect("Error starting QUIC-enabled Host");
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Quic, Idle, Pose> = NodeConfig::new("pose").build().unwrap();
    let node = node.activate().unwrap();

    for i in 0..5 {
        // Could get this by reading a GPS, for example
        let pose = Pose {
            x: i as f32,
            y: i as f32,
        };

        node.publish(pose.clone()).unwrap();
        thread::sleep(Duration::from_millis(10));
        let result = node.request().unwrap();
        println!("Got position: {:?}", result);

        assert_eq!(pose, result.data);
    }
}
