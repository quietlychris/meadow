#![deny(unused_must_use)]

use meadow::*;
mod common;
use common::Pose;

use serial_test::*;
use std::thread;
use std::time::Duration;

#[test]
#[serial]
fn integrate_host_and_single_node_udp() {
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Udp, Idle, Pose> = NodeConfig::new("pose").build().unwrap();
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

        assert_eq!(pose, result.data);
    }
}

#[test]
#[serial]
fn simple_udp() {
    let mut host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    println!("Started host");

    let node = NodeConfig::<Udp, f32>::new("num")
        .build()
        .unwrap()
        .activate()
        .unwrap();

    for i in 0..10 {
        let x = i as f32;

        match node.publish(x) {
            Ok(_) => (),
            Err(e) => {
                dbg!(e);
            }
        };
        thread::sleep(Duration::from_millis(1));
        let result = node.request().unwrap();
        assert_eq!(x, result.data);
    }
}

#[test]
#[serial]
fn udp_subscription() {
    let mut host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    println!("Started host");

    let node = NodeConfig::<Udp, f32>::new("num")
        .build()
        .unwrap()
        .activate()
        .unwrap();
    let subscriber = NodeConfig::<Udp, f32>::new("num")
        .build()
        .unwrap()
        .subscribe(Duration::from_millis(1))
        .unwrap();

    for i in 0..10 {
        let x = i as f32;

        node.publish(x).unwrap();
        thread::sleep(Duration::from_millis(5));
        let result = subscriber.get_subscribed_data().unwrap();
        assert_eq!(x, result.data);
    }
}
