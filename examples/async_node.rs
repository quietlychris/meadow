// use std::time::{sleep, Duration};
use tokio::time::{sleep, Duration};

use rhiza::node::{Node, NodeConfig};
use rhiza::Pose;

#[tokio::main]
async fn main() {
    // Run n publish-subscribe loops in different processes
    for i in 0..3 {
        let handle = tokio::spawn(async move {
            let thread_num = i;
            let pose = Pose {
                x: thread_num as f32,
                y: thread_num as f32,
            };

            // Create our node
            let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
            let cfg: NodeConfig<Pose> = NodeConfig::new("pose")
                .host_addr(addr);
            let mut node: Node<Pose> = Node::from_config(cfg.clone());
            node.connect().await.unwrap();

            loop {
                // Neither one of the following should compile
                // node.publish(8).await.unwrap();
                // node.publish("world".to_string()).await.unwrap();
                
                node.publish(pose.clone()).await.unwrap();
                sleep(Duration::from_millis(i)).await;
                let result = node.request().await.unwrap();
                println!("From thread {}, got: {:?}", thread_num, result);
            }
        });
    }

    loop {}
}
