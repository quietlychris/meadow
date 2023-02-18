use meadow::*;
use tracing::*;

use std::thread;
use std::time::Duration;

fn main() -> Result<(), meadow::Error> {
    tracing_subscriber::fmt()
        .compact()
        // enable everything
        .with_max_level(tracing::Level::INFO)
        // sets this to be the default, global collector for this application.
        .with_target(false)
        .init();

    let number_of_yaks = 3;
    info!(number_of_yaks, "preparing to shave yaks");

    meadow::generate_certs()?;
    let mut host: Host = HostConfig::default()
        .with_udp_config(None)
        .with_quic_config(Some(host::QuicConfig::default("lo")))
        .build()?;
    host.start()?;
    info!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Quic, Idle, String> = NodeConfig::new("TEAPOT")
        // .with_tcp_config()
        .topic("pose")
        .build()
        .unwrap();
    let node = node.activate()?;

    for i in 0..5 {
        let msg = format!("Hello #{}", i);
        node.publish(msg)?;
        // println!("published {}", i);
        thread::sleep(Duration::from_millis(1000));
        let result = node.request();
        info!("Received reply: {:?}", result);
    }

    println!(
        "The size of an a meadow Host before shutdown is: {}",
        std::mem::size_of_val(&host)
    );
    host.stop()?;

    Ok(())
}
