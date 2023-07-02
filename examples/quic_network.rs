use meadow::*;
use tracing::*;

use std::thread;
use std::time::Duration;

#[cfg(feature = "quic")]
fn main() -> Result<(), meadow::Error> {
    tracing_subscriber::fmt()
        .compact()
        // enable everything
        .with_max_level(tracing::Level::INFO)
        // sets this to be the default, global collector for this application.
        .with_target(false)
        .init();

    meadow::generate_certs()?;
    let mut host: Host = HostConfig::default()
        .with_tcp_config(None)
        .with_udp_config(None)
        .with_quic_config(Some(host::QuicConfig::default()))
        .build()?;
    host.start()?;
    info!("Host should be running in the background");

    // Get the writer up and running
    let node = NodeConfig::<Quic, String>::new("WRITER")
        .topic("pose")
        .build()?
        .activate()?;

    // Create a subscription node with a query rate of 10 Hz
    let reader = NodeConfig::<Quic, String>::new("READER")
        .topic("pose")
        .build()?
        .subscribe(Duration::from_millis(50))?;

    for i in 0..5 {
        let msg = format!("Hello #{}", i);
        node.publish(msg)?;
        info!("Published message #{}", i);
        // println!("published {}", i);
        let value = node.request().unwrap();
        thread::sleep(Duration::from_millis(100));
        info!("Received reply: {:?}", reader.get_subscribed_data());
    }

    println!(
        "The size of an a meadow Host before shutdown is: {}",
        std::mem::size_of_val(&host)
    );
    host.stop()?;

    Ok(())
}

#[cfg(not(feature = "quic"))]

fn main() {
    panic!("Must enable the \"quic\" feature to run");
}
