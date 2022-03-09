use bissel::*;
use std::error::Error;

use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    //let sled_cfg = SledConfig::default().path("store").temporary(true);
    //let tcp_cfg = TcpConfig::default("lo");
    //let udp_cfg = UdpConfig::default("lo");
    let mut host = HostConfig::default()
        .with_sled_config(SledConfig::default().path("store").temporary(true))
        // .with_tcp_config(Some(TcpConfig::default("wlp3s0")))
        // .with_udp_config(None)
        .build()?;
    host.start()?;
    println!("Started host");

    let node_thread = thread::spawn(|| {
        let node = NodeConfig::new("TEAPOT")
            .topic("num")
            .build()
            .unwrap()
            .connect()
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
    println!("And now we're here");
    let node = NodeConfig::<f32>::new("bloop")
        .topic("num")
        .build()
        .unwrap()
        .connect()
        .unwrap();
    println!("Built second node");
    thread::sleep(Duration::from_millis(1000));
    for i in 0..30 {
        thread::sleep(Duration::from_millis(200));
        let result = node.request().unwrap();
        dbg!(result);
    }

    thread::sleep(Duration::from_millis(10_000));
    node_thread.join().unwrap();
    Ok(())
}
