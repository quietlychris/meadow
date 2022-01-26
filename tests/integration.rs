#![deny(unused_must_use)]
use std::error::Error;

use rhiza::host::*;
use rhiza::node::*;
use rhiza::Pose;

use std::thread;
use std::time::Duration;

#[test]
fn integrate_host_and_single_node() {
    let mut host: Host = HostConfig::new("lo")
        .socket_num(25_000)
        .store_filename("store")
        .build()
        .unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let mut node: Node<Pose> = NodeConfig::new("TEST_NODE")
        .topic("pose")
        .build()
        .unwrap();
    node.connect().unwrap();

    for i in 0..5 {
        // Could get this by reading a GPS, for example
        let pose = Pose {
            x: i as f32,
            y: i as f32,
        };

        node.publish(pose.clone()).unwrap();
        thread::sleep(Duration::from_millis(1_000));
        let result = node.request().unwrap();
        println!("Got position: {:?}", result);

        assert_eq!(pose, result);
    }

    host.stop().unwrap();
}

#[test]
fn request_non_existent_topic() {
    let mut host: Host = HostConfig::new("lo")
        .socket_num(25_000)
        .store_filename("store")
        .build()
        .unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let mut node: Node<Pose> = NodeConfig::new("TEST_NODE")
        .topic("doesnt_exist")
        .build()
        .unwrap();
    node.connect().unwrap();

    // Requesting a topic that doesn't exist should return a recoverable error
    for i in 0..5 {
        println!("on loop: {}", i);
        let result  = node.request();
        dbg!(&result);
        thread::sleep(Duration::from_millis(50));
    }

    host.stop().unwrap();
}

#[test]
fn publish_boolean() {
    let mut host: Host = HostConfig::new("lo")
        .socket_num(25_000)
        .store_filename("store")
        .build()
        .unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let mut node: Node<bool> = NodeConfig::new("TEST_NODE")
        .topic("my_boolean")
        .build()
        .unwrap();
    node.connect().unwrap();

    for i in 0..5 {
        node.publish(true).unwrap();
        thread::sleep(Duration::from_millis(50));
        assert_eq!(true, node.request().unwrap());
    }

    host.stop().unwrap();
}
