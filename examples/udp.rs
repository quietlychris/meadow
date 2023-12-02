use meadow::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), meadow::Error> {
    start_logging();
    let mut host = HostConfig::default()
        .with_udp_config(Some(host::UdpConfig::default("lo")))
        .with_tcp_config(None)
        .build()?;
    host.start()?;
    println!("Started host");

    let node = NodeConfig::<Udp, f32>::new("num")
        .build()
        .unwrap()
        .activate()?;
    node.publish(0 as f32)?;


    let subscriber = NodeConfig::<Udp, f32>::new("num")
        .build()
        .unwrap()
        .subscribe(Duration::from_millis(1))?;

    for i in 1..5 {
        node.publish(i as f32)?;
        thread::sleep(Duration::from_millis(10));
        let result = node.request()?;
        //dbg!(node.topics()?);
        // dbg!(result);
        dbg!(subscriber.get_subscribed_data()?);
    }

    host.stop()?;

    Ok(())
}

fn start_logging() {
    let file_appender = tracing_appender::rolling::hourly("logs/", "subscription");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();
}
