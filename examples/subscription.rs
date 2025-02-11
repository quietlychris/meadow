use meadow::prelude::*;
use std::time::Duration;

fn main() -> Result<(), meadow::Error> {
    // Set up logging
    logging();

    type N = Udp;

    let mut host: Host = HostConfig::default()
        //.with_udp_config(None)
        // .with_tcp_config(None)
        .build()?;
    host.start()?;

    // Get the host up and running
    let writer = NodeConfig::<Blocking, N, _>::new("subscription")
        .build()?
        .activate()?;

    // Create a subscription node with a query rate of 10 Hz
    let reader = NodeConfig::<Blocking, N, usize>::new("subscription")
        .build()?
        .subscribe(Duration::from_millis(500))?;

    // Since subscribed topics are not guaranteed to exist, subscribed nodes always return Option<T>
    //let _result = reader.get_subscribed_data();
    //dbg!(_result);

    for i in 0..5usize {
        println!("publishing {}", i);
        writer.publish(i).unwrap();
        println!("Published");
        std::thread::sleep(std::time::Duration::from_millis(100));
        println!("About to request");
        let result = writer.request().unwrap().data;
        dbg!(result);
        //assert_eq!(writer.request()?.data, i);
        //assert_eq!(reader.get_subscribed_data()?.data, i);
    }

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
        .with_ansi(true)
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
