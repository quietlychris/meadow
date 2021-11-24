// use std::time::{sleep, Duration};
use tokio::time::{sleep, Duration};

use rhiza::node::Node;
use rhiza::Pose;

#[tokio::main]
async fn main() {
    let mut node: Node<Pose> = Node::default("hello");
    node.interface("lo").await.unwrap();
    println!("connected");
    let pose = Pose { x: 2.0, y: 20.0 };
    //println!("About to publish");
    // node.publish(8).await.unwrap();
    // node.publish("world".to_string()).await.unwrap();
    node.publish(pose).await.unwrap();
    sleep(Duration::from_millis(2_000)).await;
    let result = node.request().await.unwrap();
    println!("{:?}", result);

    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let thread_num = i;
            let mut node: Node<Pose> = Node::default("hello");
            node.interface("lo").await.unwrap();
            loop {
                let result = node.request().await.unwrap();
                sleep(Duration::from_millis(20 * thread_num)).await;
                println!("From thread {}, got: {:?}", thread_num, result);
            }
        });
    }

    loop {}

    /*
    let mut node: Node<Vec<u8>> = Node::default("hello");
    node.interface("lo").await.unwrap();
    let data = vec![1,2,3,4];
    node.publish(data).await.unwrap();
    */

    /*
    loop {
        let pose = Pose { x: 1.0, y: 20.0 };
        println!("About to publish");
        node.publish(pose).await.unwrap();
        println!("Successfully published");
        sleep(Duration::from_millis(2_000)).await;
    }*/
}
