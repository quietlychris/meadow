use rhiza::node::{Node, NodeConfig};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Coordinate {
    x: f32,
    y: f32,
}

fn main() {
    // let addr = "192.168.8.105:25000"
    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let mut node: Node<Coordinate> = NodeConfig::new("pose").host_addr(addr).build().unwrap();
    node.connect().unwrap();

    let c = Coordinate { x: 4.0, y: 4.0 };
    node.publish_to("pose", c).unwrap();

    loop {
        // Could get this by reading a GPS, for example
        let c = Coordinate { x: 4.0, y: 4.0 };
        node.publish_to("pose", c).unwrap();
        thread::sleep(Duration::from_millis(1_000));
        let result: Coordinate = node.request("pose").unwrap();
        println!("Got position: {:?}", result);
    }
}
