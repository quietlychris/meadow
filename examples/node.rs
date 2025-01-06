use meadow::node::config::NodeConfig;
use meadow::node::network_config::{Blocking, NetworkConfig, Tcp};
use meadow::node::{Idle, Node};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Coordinate {
    x: f32,
    y: f32,
}

fn main() -> Result<(), meadow::Error> {
    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let node: Node<Blocking, Tcp, Idle, Coordinate> = NodeConfig::new("my_coordinate")
        .with_config(NetworkConfig::<Blocking, Tcp>::default().set_host_addr(addr))
        .build()
        .unwrap();
    let node = node.activate()?; // unwrap();

    let c = Coordinate { x: 4.0, y: 4.0 };
    node.publish(c)?;

    for i in 0..5 {
        // Could get this by reading a GPS, for example
        let c = Coordinate {
            x: i as f32,
            y: i as f32,
        };
        node.publish(c)?;
        thread::sleep(Duration::from_millis(1_000));
        let result = node.request()?;
        println!("Got coordinate: {:?}", result);
    }
    Ok(())
}
