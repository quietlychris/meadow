use bissel::*;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Coordinate {
    x: f32,
    y: f32,
}

fn main() {
    // let addr = "192.168.8.105:25000"
    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let node: Node<Idle, Coordinate> = NodeConfig::new("SIMPLE_NODE")
        .topic("my_coordinate")
        .with_tcp_config(node::TcpConfig::default().set_host_addr(addr))
        .build()
        .unwrap();
    let node = node.activate().unwrap();

    let c = Coordinate { x: 4.0, y: 4.0 };
    node.publish(c).unwrap();

    loop {
        // Could get this by reading a GPS, for example
        let c = Coordinate { x: 4.0, y: 4.0 };
        node.publish(c).unwrap();
        thread::sleep(Duration::from_millis(1_000));
        let result = node.request().unwrap();
        println!("Got coordinate: {:?}", result);
    }
}
