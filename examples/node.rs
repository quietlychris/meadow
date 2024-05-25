use meadow::*;
use std::thread;
use tokio::time;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Coordinate {
    x: f32,
    y: f32,
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let node: Node<Tcp, Idle, Coordinate> = NodeConfig::new("my_coordinate")
        .with_config(node::nonblocking::NetworkConfig::<Tcp>::default().set_host_addr(addr))
        .build()
        .unwrap();
    let node = node.activate().await.unwrap(); // unwrap();

    let c = Coordinate { x: 4.0, y: 4.0 };
    node.publish(c).await.unwrap();

    for i in 0..5 {
        // Could get this by reading a GPS, for example
        let c = Coordinate { x: i as f32, y: i as f32 };
        node.publish(c).await.unwrap();
        time::sleep(Duration::from_millis(1_000)).await;
        let result = node.request().await.unwrap();
        println!("Got coordinate: {:?}", result);
    }
}
