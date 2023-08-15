use std::{fs::File, sync::Arc};
use tracing::*;
use tracing_subscriber::{filter, prelude::*};

#[cfg(feature = "quic")]
use meadow::host::quic::generate_certs;

#[cfg(feature = "quic")]
use meadow::node::Quic;

#[cfg(feature = "quic")]
fn main() -> Result<(), meadow::Error> {
    use meadow::*;
    use std::thread;
    use std::time::Duration;
    use tracing::*;

    logging();

    generate_certs()?;
    let mut host: Host = HostConfig::default()
        .with_tcp_config(None)
        .with_udp_config(None)
        .with_quic_config(Some(host::QuicConfig::default()))
        .build()?;
    host.start()?;
    debug!("Host should be running in the background");

    // Get the writer up and running
    let node = NodeConfig::<Quic, String>::new("pose")
        .build()?
        .activate()?;

    // Create a subscription node with a query rate of 10 Hz
    let reader = NodeConfig::<Quic, String>::new("pose")
        .build()?
        .subscribe(Duration::from_millis(50))?;

    for i in 0..5 {
        let msg = format!("Hello #{}", i);
        node.publish(msg)?;
        //debug!("Published message #{}", i);
        println!("published {}", i);
        let value = node.request().unwrap();
        println!("QUIC request received with value {:?}", value);
        thread::sleep(Duration::from_millis(100));
        println!("Received reply: {:?}", reader.get_subscribed_data());
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

fn logging() {
    // A layer that logs events to a file.
    let file = File::create("logs/debug.log");
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Error: {:?}", error),
    };

    let log = tracing_subscriber::fmt::layer()
        .compact()
        .with_line_number(true)
        .with_writer(Arc::new(file));

    tracing_subscriber::registry()
        .with(
            log
                // Add an `INFO` filter to the stdout logging layer
                .with_filter(filter::LevelFilter::DEBUG),
        )
        .init();
}
