#![deny(unused_must_use)]
use serial_test::serial;
use std::error::Error;

use rhiza::host::*;
use rhiza::node::*;
use rhiza::Pose;

use std::thread;
use std::time::Duration;

#[test]
#[serial]
fn integrate_host_and_single_node() {
    let mut host: Host = HostConfig::new("lo")
        .socket_num(25_000)
        .store_filename("store")
        .build()
        .unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let mut node: Node<Pose> = NodeConfig::new("pose").host_addr(addr).build().unwrap();
    node.connect().unwrap();

    let mut result = Pose::default();

    for i in 0..5 {
        // Could get this by reading a GPS, for example
        let pose = Pose { x: i as f32, y: i as f32};

        node.publish_to("pose", pose.clone()).unwrap();
        thread::sleep(Duration::from_millis(1_000));
        result = node.request("pose").unwrap();
        println!("Got position: {:?}", result);

        assert_eq!(pose, result);
    }

    host.stop().unwrap();
}

#[test]
#[serial]
fn request_non_existent_topic() {
    let mut host: Host = HostConfig::new("lo")
        .socket_num(25_000)
        .store_filename("store")
        .build()
        .unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let mut node: Node<Pose> = NodeConfig::new("pose").host_addr(addr).build().unwrap();
    node.connect().unwrap();

    for i in 0..5 {
        println!("on loop: {}", i);
        let result: Result<Pose, Box<postcard::Error>> = node.request("doesnt_exist");
        dbg!(&result);
        thread::sleep(Duration::from_millis(50));
    }

    host.stop().unwrap();
}
