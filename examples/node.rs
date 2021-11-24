// use std::time::{sleep, Duration};
use tokio::time::{sleep, Duration};

use rhiza::node::Node;
use rhiza::Pose;

#[tokio::main]
async fn main() {
    let mut node: Node<Pose> = Node::default("hello");
    node.interface("lo").await.unwrap();

    // Neither one of the following should compile
    // node.publish(8).await.unwrap();
    // node.publish("world".to_string()).await.unwrap();

    // Run a single publish-subscribe loop
    /*
    loop {
        node.publish(pose.clone()).await.unwrap();
        sleep(Duration::from_millis(1_000)).await;
        let result = node.request().await.unwrap();
        println!("Received back: {:?}", result);
    }
    */

    // Run n publish-subscribe loops in different processes
    for i in 0..1 {
        let handle = tokio::spawn(async move {
            let thread_num = i;
            let mut node: Node<Pose> = Node::default("hello");
            let pose = Pose { x: thread_num as f32, y: thread_num as f32 };
            node.interface("lo").await.unwrap();
            loop {
                node.publish(pose.clone()).await.unwrap();
                sleep(Duration::from_millis(1_000)).await;
                let result = node.request().await.unwrap();
                println!("From thread {}, got: {:?}", thread_num, result);
            }
        });
    }

    loop {}
}
