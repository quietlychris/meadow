// use std::time::{sleep, Duration};
use rhiza::host::*;
use rhiza::node::{Node, NodeConfig};
use rhiza::Pose;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

const LABELS: [&str; 9] = [
    "all",
    "along",
    "the_1",
    "watchtower",
    "the_2",
    "princes",
    "kept",
    "the_3",
    "view",
];

#[tokio::main]
async fn main() {
    let mut host: Host = HostConfig::new("lo")
        .socket_num(25_000)
        .store_filename("store")
        .build()
        .unwrap();
    host.start().await.unwrap();

    // Run n publish-subscribe loops in different processes
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    for i in 0..LABELS.len() {
        let handle = tokio::spawn(async move {
            let thread_num = i;
            let pose = Pose {
                x: thread_num as f32,
                y: thread_num as f32,
            };

            // Create our node
            let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
            let mut node: Node<Pose> = NodeConfig::new(LABELS[i]).host_addr(addr).build();
            node.connect().await.unwrap();
            // sleep(Duration::from_millis(1)).await;

            loop {
                node.publish_to("pose", pose.clone()).await.unwrap();
                sleep(Duration::from_millis(i as u64)).await;
                let result: Pose = node.request("pose").await.unwrap();
                println!("From thread {}, got: {:?}", thread_num, result);
            }
        });
        handles.push(handle);
    }

    sleep(Duration::from_secs(3)).await;

    for handle in handles {
        handle.abort();
    }
    host.stop().unwrap();
}
