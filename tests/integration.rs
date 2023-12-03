#![deny(unused_must_use)]

use meadow::*;

use std::thread;
use std::time::Duration;

#[cfg(feature = "quic")]
use std::sync::Once;

#[cfg(feature = "quic")]
use meadow::host::quic::generate_certs;

#[cfg(feature = "quic")]
static INIT: Once = Once::new();

#[cfg(feature = "quic")]
pub fn initialize() {
    INIT.call_once(|| {
        generate_certs().unwrap();
    });
}

/// Example test struct for docs and tests
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
#[repr(C)]
struct Pose {
    pub x: f32,
    pub y: f32,
}

/// Example test struct for docs and tests, incompatible with Pose
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
struct NotPose {
    a: isize,
}

#[test]
fn integrate_host_and_single_node() {
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Tcp, Idle, Pose> = NodeConfig::new("pose").build().unwrap();
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
fn request_non_existent_topic() {
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Tcp, Idle, Pose> = NodeConfig::new("doesnt_exist").build().unwrap();
    let node = node.activate().unwrap();

    // Requesting a topic that doesn't exist should return a recoverable error
    for i in 0..5 {
        println!("on loop: {}", i);
        let result = node.request();
        dbg!(&result);
        thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn node_send_options() {
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();

    // Get the host up and running
    let node_a = NodeConfig::<Tcp, Option<f32>>::new("pose")
        .build()
        .unwrap()
        .activate()
        .unwrap();
    let node_b = NodeConfig::<Tcp, Option<f32>>::new("pose")
        .build()
        .unwrap()
        .activate()
        .unwrap();

    // Send Option with `Some(value)`
    node_a.publish(Some(1.0)).unwrap();
    let result = node_b.request().unwrap();
    dbg!(&result);
    assert_eq!(result.data.unwrap(), 1.0);

    // Send option with `None`
    node_a.publish(None).unwrap();
    let result = node_b.request();
    dbg!(&result);
    assert_eq!(result.unwrap().data, None);
}

#[test]
fn publish_boolean() {
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Tcp, Idle, bool> = NodeConfig::new("my_boolean").build().unwrap();
    let node = node.activate().unwrap();

    for _i in 0..5 {
        node.publish(true).unwrap();
        thread::sleep(Duration::from_millis(50));
        assert!(node.request().unwrap().data);
    }

    host.stop().unwrap();
}

#[test]
fn subscription_usize() {
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let writer = NodeConfig::<Tcp, usize>::new("subscription")
        .build()
        .unwrap()
        .activate()
        .unwrap();

    // Create a subscription node with a query rate of 100 Hz
    let reader = writer
        .cfg
        .clone()
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
            Ok(result) => assert_eq!(test_value, result.data),
            Err(e) => println!("{:?}", e),
        }
        // dbg!(result);
    }
}

#[test]
#[should_panic]
fn no_subscribed_value() {
    let mut host: Host = HostConfig::default().build().unwrap();
    host.start().unwrap();

    // Create a subscription node with a query rate of 10 Hz
    let reader = NodeConfig::<Tcp, usize>::new("subscription")
        .build()
        .unwrap()
        .subscribe(Duration::from_millis(100))
        .unwrap();

    // Unwrapping on an error should lead to panic
    let _result: usize = reader.get_subscribed_data().unwrap().data;
}

#[test]
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

        match node.publish(x) {
            Ok(_) => (),
            Err(e) => {
                dbg!(e);
            }
        };
        thread::sleep(Duration::from_millis(5));
        let result = subscriber.get_subscribed_data().unwrap();
        assert_eq!(x, result.data);
    }
}
