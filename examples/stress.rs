use meadow::prelude::*;
use rand::Rng;
// use rayon::par_iter;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), meadow::Error> {
    let mut host = HostConfig::default()
        .with_sled_config(
            SledConfig::default()
                .temporary(false)
                .path("./logs/stress.sled"),
        )
        .build()?;
    host.start()?;

    let duration = Duration::from_secs(10);
    let n = 10;

    let tcp = thread::spawn(move || {
        run_tcp(n, duration).unwrap();
    });

    let udp = thread::spawn(move || {
        run_udp(n, duration).unwrap();
    });

    thread::sleep(duration);

    if let Err(e) = tcp.join() {
        println!("TCP error: {:?}", e);
    }

    if let Err(e) = udp.join() {
        println!("UDP error: {:?}", e);
    }

    let topics = host.topics();
    for topic in &topics {
        let db = host.store.clone();
        let tree = db.open_tree(topic.as_bytes()).unwrap();
        println!("Topic {} has {} stored values", topic, tree.len());
    }

    Ok(())
}

fn run_tcp(n: usize, duration: Duration) -> Result<(), meadow::Error> {
    let mut nodes = Vec::with_capacity(n);
    for i in 0..n {
        let topic = format!("tcp_{}", i);
        let node = NodeConfig::<Blocking, Tcp, f32>::new(topic).build()?;
        let node = node.activate()?;
        nodes.push(node);
    }

    let start = Instant::now();
    let mut rng = rand::thread_rng();
    while start.elapsed() < duration {
        for node in &mut nodes {
            node.publish(rng.gen())?;
            node.request()?;
        }
    }
    Ok(())
}

fn run_udp(n: usize, duration: Duration) -> Result<(), meadow::Error> {
    let mut nodes = Vec::with_capacity(n);
    for i in 0..n {
        let topic = format!("udp_{}", i);
        let node = NodeConfig::<Blocking, Udp, f32>::new(topic).build()?;
        let node = node.activate()?;
        nodes.push(node);
    }

    let start = Instant::now();
    let mut rng = rand::thread_rng();
    while start.elapsed() < duration {
        for node in &mut nodes {
            node.publish(rng.gen())?;
            node.request()?;
        }
    }
    Ok(())
}
