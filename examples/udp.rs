use meadow::prelude::*;
use std::thread;
use std::time::Duration;
// For logging
use std::{fs::File, sync::Arc};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{filter, prelude::*};

fn main() -> Result<(), meadow::Error> {
    logging();
    let mut host = {
        let date = chrono::Utc::now();
        let stamp = format!(
            "{}_{}_UTC",
            date.date_naive(),
            date.time().format("%H:%M:%S")
        );
        let sled_cfg = SledConfig::default()
            .path(format!("./logs/{}", stamp))
            // If we wanted to keep the logs, we'd make this `false`
            .temporary(true);
        HostConfig::default()
            .with_udp_config(Some(UdpConfig::default("lo")))
            .with_tcp_config(None)
            .with_sled_config(sled_cfg)
            .build()?
    };
    host.start()?;
    println!("Started host");

    let node = NodeConfig::<Blocking, Udp, f32>::new("num")
        .build()
        .unwrap()
        .activate()?;
    node.publish(0 as f32)?;

    let subscriber = NodeConfig::<Blocking, Udp, f32>::new("num")
        .build()
        .unwrap()
        .subscribe(Duration::from_millis(1))?;

    for i in 1..5 {
        node.publish(i as f32)?;
        thread::sleep(Duration::from_millis(10));
        let _result = node.request()?;
        //dbg!(node.topics()?);
        // dbg!(result);
        dbg!(subscriber.get_subscribed_data()?);
    }

    host.stop()?;

    Ok(())
}

fn logging() {
    // A layer that logs events to a file.
    let file = File::create("logs/debug.log");
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Error: {:?}", error),
    };

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()
        .unwrap()
        .add_directive("meadow=info".parse().unwrap());
    // .add_directive("sled=none".parse().unwrap());

    let log = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(false)
        .with_line_number(true)
        .with_target(false)
        .with_writer(Arc::new(file));

    tracing_subscriber::registry()
        .with(
            log
                // Add an `INFO` filter to the stdout logging layer
                // sled logs tons of stuff with DEBUG and TRACE levels
                .with_filter(filter::LevelFilter::INFO) // .with_filter(filter::LevelFilter::WARN)
                .with_filter(filter), //.with_filter(filter::LevelFilter::ERROR)
        )
        .init();
}
