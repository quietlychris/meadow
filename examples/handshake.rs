use std::error::Error;

use rhiza::host::*;
use rhiza::node::*;
use rhiza::Pose;

use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let mut host: Host = HostConfig::new("lo")
        .socket_num(25_000)
        .store_filename("store")
        .build()
        .unwrap();
    host.start().await.unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let mut node: Node<Pose> = NodeConfig::new("pose").host_addr(addr).build();
    node.connect().await.unwrap();

    let mut result = Pose::default();

    // Could get this by reading a GPS, for example
    let pose = Pose { x: 4.0, y: 4.0 };

    node.publish_to("pose", pose.clone()).await.unwrap();
    sleep(Duration::from_millis(1_000)).await;
    result = node.request("pose").await.unwrap();
    println!("Got position: {:?}", result);

    assert_eq!(pose, result);
    host.stop().await.unwrap();
}