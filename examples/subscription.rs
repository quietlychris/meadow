use meadow::*;
use std::time::Duration;

fn main() -> Result<(), meadow::Error> {
    // Set up logging
    logging();

    type N = Quic;

    let mut host: Host = HostConfig::default().with_udp_config(None).build()?;
    host.start()?;
    println!("Host should be running in the background");

    // Get the host up and running
    let writer = NodeConfig::<N, _>::new("subscription")
        .build()?
        .activate()?;

    // Create a subscription node with a query rate of 10 Hz
    let reader = writer
        .cfg
        .clone()
        .build()?
        .subscribe(Duration::from_micros(1))?;

    // Since subscribed topics are not guaranteed to exist, subscribed nodes always return Option<T>
    let result = match reader.get_subscribed_data() {
        Ok(val) => val.data,
        Err(e) => {
            println!("Error: {:?}, returning 0", e);
            0
        }
    };
    dbg!(result);

    for i in 0..100 {
        println!("publishing {}", i);
        writer.publish(i as usize)?;
        std::thread::sleep(std::time::Duration::from_micros(200));
        match reader.get_subscribed_data() {
            Ok(result) => {
                dbg!(result);
            }
            Err(e) => {
                dbg!(e);
            }
        };
    }

    // host.stop()?;
    Ok(())
}

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
        .with_ansi(false)
        .with_line_number(true)
        .with_writer(Arc::new(file));

    tracing_subscriber::registry()
        .with(
            log
                // Add an `INFO` filter to the stdout logging layer
                .with_filter(filter::LevelFilter::INFO), // .with_filter(filter::LevelFilter::WARN)
                                                         // .with_filter(filter::LevelFilter::ERROR)
        )
        .init();
}
