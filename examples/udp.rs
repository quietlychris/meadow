use bissel::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut host = HostConfig::default()
        .with_sled_config(SledConfig::default().path("store").temporary(true))
        // .with_tcp_config(Some(TcpConfig::default("wlp3s0")))
        .with_udp_config(Some(host::UdpConfig::default("wlp3s0")))
        .build()?;
    host.start()?;

    let node_thread = thread::spawn(|| {
        let udp_socket = "192.168.8.105:25000"
            .parse::<std::net::SocketAddr>()
            .unwrap();
        let udp_cfg: node::UdpConfig = node::UdpConfig::new(udp_socket).unwrap();
        let node = NodeConfig::new("SENDER")
            .with_udp_config(udp_cfg)
            .topic("num")
            .build()
            .unwrap()
            .activate()
            .unwrap();
        println!("Built first node");
        for i in 0..10 {
            let x = i as f32;

            node.publish_udp(x).unwrap();
            println!("published {} over udp", i);
            thread::sleep(Duration::from_millis(1000));
        }
        std::process::exit(0);
    });

    let node = NodeConfig::<f32>::new("RECEIVER")
        .topic("num")
        .build()
        .unwrap()
        .activate()?;

    thread::sleep(Duration::from_millis(1000));
    for i in 0..30 {
        thread::sleep(Duration::from_millis(500));
        let result = node.request()?;
        dbg!(result);
    }

    thread::sleep(Duration::from_millis(10_000));
    node_thread.join().unwrap();
    Ok(())
}
