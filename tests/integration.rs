#![deny(unused_must_use)]

use meadow::host::*;
use meadow::node::*;
use meadow::Pose;

use std::thread;
use std::time::Duration;

#[test]
fn integrate_host_and_single_node() {
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Idle, Pose> = NodeConfig::new("TEST_NODE").topic("pose").build().unwrap();
    let node = node.activate().unwrap();

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
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Idle, Pose> = NodeConfig::new("TEST_NODE")
        .topic("doesnt_exist")
        .build()
        .unwrap();
    let node = node.activate().unwrap();

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
fn node_send_options() {
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let node_a = NodeConfig::<Option<f32>>::new("OptionTx")
        .topic("pose")
        .build()
        .unwrap()
        .activate()
        .unwrap();
    let node_b = NodeConfig::<Option<f32>>::new("OptionTx")
        .topic("pose")
        .build()
        .unwrap()
        .activate()
        .unwrap();

    // Send Option with `Some(value)`
    node_a.publish(Some(1.0)).unwrap();
    assert_eq!(node_b.request().unwrap().unwrap(), 1.0);

    // Send option with `None`
    node_a.publish(None).unwrap();
    assert_eq!(node_b.request().unwrap(), None);

    host.stop().unwrap();
}

#[test]
fn publish_boolean() {
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Idle, bool> = NodeConfig::new("TEST_NODE")
        .topic("my_boolean")
        .build()
        .unwrap();
    let node = node.activate().unwrap();

    for _i in 0..5 {
        node.publish(true).unwrap();
        thread::sleep(Duration::from_millis(50));
        assert!(node.request().unwrap());
    }

    host.stop().unwrap();
}

#[test]
fn subscription_usize() {
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let writer = NodeConfig::new("WRITER")
        .topic("subscription")
        .build()
        .unwrap()
        .activate()
        .unwrap();

    // Create a subscription node with a query rate of 100 Hz
    let reader = writer
        .cfg
        .clone()
        .name("READER")
        .build()
        .unwrap()
        .subscribe(Duration::from_millis(10))
        .unwrap();

    for i in 0..5 {
        let test_value = i as usize;
        writer.publish(test_value).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        // let result = reader.get_subscribed_data();
        match reader.get_subscribed_data() {
            Ok(result) => assert_eq!(test_value, result),
            Err(e) => println!("{:?}", e),
        }
        // dbg!(result);
    }

    // host.stop().unwrap();
}

#[test]
#[should_panic]
fn no_subscribed_value() {
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();

    // Create a subscription node with a query rate of 10 Hz
    let reader = NodeConfig::<usize>::new("READER")
        .topic("subscription")
        .build()
        .unwrap()
        .subscribe(Duration::from_millis(100))
        .unwrap();

    let _result: usize = reader.get_subscribed_data().unwrap();
}
