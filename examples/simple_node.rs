use rhiza::node::{Node, NodeConfig};
use tokio::time::{sleep, Duration};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Coordinate {
    x: f32,
    y: f32,
}

#[tokio::main]
async fn main() {
    let addr = "192.168.8.105:25000"
        .parse::<std::net::SocketAddr>()
        .unwrap();
    let cfg: NodeConfig<Coordinate> = NodeConfig::new("pose").host_addr(addr);
    let mut node: Node<Coordinate> = Node::from_config(cfg);
    node.connect().await.unwrap();

    loop {
        // Could get this by reading a GPS, for example
        let c = Coordinate { x: 4.0, y: 4.0 };

        node.publish(c).await.unwrap();
        sleep(Duration::from_millis(1_000)).await;
        let result = node.request().await.unwrap();
        println!("Got position: {:?}", result);
    }
}
