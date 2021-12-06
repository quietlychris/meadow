use rhiza::host::{Host, HostConfig};
use rhiza::node::{Node, NodeConfig};
use tokio::time::{sleep, Duration};

use rand::prelude::*;

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
struct Coordinate {
    x: f32,
    y: f32,
}

#[tokio::main]
async fn main() {
    let mut rng = rand::thread_rng();
    // Seed host with a known value
    {
        let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
        let cfg: NodeConfig<Coordinate> = NodeConfig::new("pose").host_addr(addr);
        let mut node: Node<Coordinate> = Node::from_config(cfg);
        node.connect().await.unwrap();
        let c = Coordinate { x: 4.0, y: 4.0 };
        node.publish_to("pose", c).await.unwrap();
    }

    let subscribed_val = Arc::new(Mutex::new(Coordinate::default()));
    let svc = Arc::clone(&subscribed_val);

    let subscription = tokio::spawn(async move {
        let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
        let cfg: NodeConfig<Coordinate> = NodeConfig::new("pose").host_addr(addr);
        let mut node: Node<Coordinate> = Node::from_config(cfg);
        node.connect().await.unwrap();

        loop {
            let mut result = svc.lock().await;
            let request: Coordinate = node.request("pose").await.unwrap();
            *result = request;
            sleep(Duration::from_millis(1_000)).await;
        }
    });

    // This value will update at regular intervals
    for i in 0..10 {
        println!("Got position: {:?}", subscribed_val.lock().await);
        sleep(Duration::from_millis(1_000)).await;
    }
}

fn create_subscription<T: rhiza::Message>(node: Node<T>, val: Arc<Mutex<T>>, interval: Duration) {}
