use bissel::*;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    start_logging();

    let mut host: Host = HostConfig::default().build()?;
    host.start()?;
    println!("Host should be running in the background");

    // Get the host up and running
    let writer = NodeConfig::new("WRITER")
        .topic("subscription")
        .build()?
        .activate()?;

    // Create a subscription node with a query rate of 10 Hz
    let reader = writer
        .cfg
        .clone()
        .name("READER")
        .build()?
        .subscribe(Duration::from_millis(100))?;

    // Since subscribed topics are not guaranteed to exist, subscribed nodes always return Option<T>
    let result = reader.get_subscribed_data()?;
    dbg!(result);

    for i in 0..5 {
        println!("publishing {}", i);
        writer.publish(i as usize)?;
        std::thread::sleep(std::time::Duration::from_millis(250));
        let result = reader.get_subscribed_data()?;
        dbg!(result);
    }

    host.stop()?;
    Ok(())
}

fn start_logging() {
    let file_appender = tracing_appender::rolling::hourly("logs/", "subscription");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();
}
