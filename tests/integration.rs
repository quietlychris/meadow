#![deny(unused_must_use)]

use bissel::host::*;
use bissel::node::*;
use bissel::Pose;

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
    let node: Node<Idle, Pose> = NodeConfig::new("TEST_NODE").topic("pose").build().unwrap();
    let mut node = node.connect().unwrap();

    for i in 0..5 {
        // Could get this by reading a GPS, for example
        let pose = Pose {
            x: i as f32,
            y: i as f32,
        };

        node.publish(pose.clone()).unwrap();
        thread::sleep(Duration::from_millis(10));
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
    let node: Node<Idle, Pose> = NodeConfig::new("TEST_NODE")
        .topic("doesnt_exist")
        .build()
        .unwrap();
    let mut node = node.connect().unwrap();

    // Requesting a topic that doesn't exist should return a recoverable error
    for i in 0..5 {
        println!("on loop: {}", i);
        let result = node.request();
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
    let node: Node<Idle, bool> = NodeConfig::new("TEST_NODE")
        .topic("my_boolean")
        .build()
        .unwrap();
    let mut node = node.connect().unwrap();

    for _i in 0..5 {
        node.publish(true).unwrap();
        thread::sleep(Duration::from_millis(50));
        assert_eq!(true, node.request().unwrap());
    }

    host.stop().unwrap();
}

#[test]
fn subscription_usize() {
    let mut host: Host = HostConfig::new("lo").build().unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let writer = NodeConfig::new("WRITER")
        .topic("subscription")
        .build()
        .unwrap()
        .connect()
        .unwrap();

    // Create a subscription node with a query rate of 10 Hz
    let mut reader = writer
        .rebuild_config()
        .name("READER")
        .build()
        .unwrap()
        .subscribe(Duration::from_millis(20))
        .unwrap();

    for i in 0..5 {
        let test_value = i as usize;
        writer.publish(test_value).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(250));
        let result = reader.get_subscribed_data().unwrap().unwrap();
        dbg!(result);
        assert_eq!(test_value, result);
    }

    // host.stop().unwrap();
}
