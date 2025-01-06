#[cfg(feature = "quic")]
fn main() -> Result<(), meadow::Error> {
    use meadow::host::quic::QuicCertGenConfig;
    use meadow::prelude::*;
    use std::thread;
    use std::time::Duration;
    use tracing::*;

    logging();

    generate_certs(QuicCertGenConfig::default());
    let mut host: Host = HostConfig::default()
        .with_tcp_config(None)
        .with_udp_config(None)
        .with_quic_config(Some(QuicConfig::default()))
        .build()?;
    host.start()?;
    debug!("Host should be running in the background");

    // Get the writer up and running
    let node = NodeConfig::<Blocking, Quic, usize>::new("pose")
        .build()?
        .activate()?;

    // Create a subscription node with a query rate of 10 Hz
    let reader = NodeConfig::<Blocking, Quic, usize>::new("pose")
        .build()?
        .subscribe(Duration::from_millis(50))?;

    for i in 0..5 {
        node.publish(i)?;
        println!("Published {}", i);
        let value = node.request()?;
        assert_eq!(i, value.data);
        println!("QUIC request received with value {:?}", value);
        dbg!(node.topics()?);
        thread::sleep(Duration::from_millis(100));
        println!("Received reply: {:?}", reader.get_subscribed_data());
    }

    println!(
        "The size of an a meadow Host before shutdown is: {}",
        std::mem::size_of_val(&host)
    );

    Ok(())
}

#[cfg(not(feature = "quic"))]

fn main() {
    panic!("Must enable the \"quic\" feature to run");
}

#[cfg(feature = "quic")]
fn logging() {
    use std::{fs::File, sync::Arc};
    use tracing_subscriber::{filter, prelude::*};

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
