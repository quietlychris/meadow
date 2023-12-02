use meadow::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), meadow::Error> {
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

    let subscriber = NodeConfig::<Udp, f32>::new("num")
        .build()
        .unwrap()
        .subscribe(Duration::from_millis(100))?;

    for i in 0..10 {
        node.publish(i as f32)?;
        thread::sleep(Duration::from_millis(50));
        let result = node.request()?;
        dbg!(node.topics()?);
        dbg!(result);
        dbg!(subscriber.get_subscribed_data());
    }

    host.stop()?;

    Ok(())
}
