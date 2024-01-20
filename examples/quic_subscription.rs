use meadow::*;
use std::thread;
use std::time::Duration;

fn main() {
    type N = Quic;

    logging();

    let mut host: Host = HostConfig::default().with_udp_config(None).build().unwrap();
    host.start().unwrap();

    // Get the host up and running
    let writer = NodeConfig::<N, usize>::new("subscription")
        .build()
        .unwrap()
        .activate()
        .unwrap();

    // Create a subscription node with a query rate of 100 Hz
    let reader = writer
        .cfg
        .clone()
        .build()
        .unwrap()
        .subscribe(Duration::from_millis(10))
        .unwrap();

    for _ in 0..3 {
        if let Err(e) = reader.get_subscribed_data() {
            println!("{:?}", e);
        } else {
            panic!("There shouldn't be a subscribed value yet...");
        };
    }


    for i in 0..5 {
        let test_value = i as usize;
        writer.publish(test_value).unwrap();
        dbg!(reader.get_subscribed_data());
        let own_back = writer.request().unwrap();
        // dbg!(own_back);
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let result = reader.get_subscribed_data();

        println!("{:?}", result);
        // dbg!(result);
    }
}

fn logging() {
    use std::{fs::File, sync::Arc};
    use tracing::*;
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
