use meadow::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), meadow::Error> {
    let mut host = HostConfig::default()
        // .with_sled_config(SledConfig::default().path("store").temporary(true))
        // .with_tcp_config(None)
        .with_udp_config(Some(host::TcpConfig::default("lo")))
        .build()?;
    host.start()?;
    println!("Started host");

    let tx_thread = thread::spawn(|| {
        let tx = NodeConfig::<Udp, f32>::new("num")
            .with_config(node::NetworkConfig::<Udp>::default())
            .build()
            .unwrap()
            .activate()
            .unwrap();
        dbg!(&tx);
        println!("Built first node");
        for i in 0..10 {
            let x = i as f32;

            match tx.publish(x) {
                Ok(_) => (),
                Err(e) => {
                    dbg!(e);
                }
            };
            println!("published {} over udp", i);
            thread::sleep(Duration::from_millis(100));
        }
        std::process::exit(0);
    });

    let rx = NodeConfig::<Tcp, f32>::new("num")
        .build()
        .unwrap()
        .activate()?;

    for _i in 0..20 {
        thread::sleep(Duration::from_millis(50));
        let result = rx.request().unwrap();
        dbg!(result);
    }

    tx_thread.join().unwrap();
    Ok(())
}
