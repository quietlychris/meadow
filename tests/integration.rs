#![deny(unused_must_use)]

use meadow::prelude::*;
mod common;
use common::Pose;
use rand::thread_rng;

use std::thread;
use std::time::Duration;

#[cfg(feature = "quic")]
use std::sync::Once;

#[cfg(feature = "quic")]
use meadow::host::quic::generate_certs;
#[cfg(feature = "quic")]
use meadow::host::quic::QuicCertGenConfig;

#[cfg(feature = "quic")]
static INIT: Once = Once::new();

#[cfg(feature = "quic")]
pub fn initialize() {
    INIT.call_once(|| {
        generate_certs(QuicCertGenConfig::default());
    });
}

fn start_host() -> Result<Host, Error> {
    let sc = SledConfig::new().temporary(true);
    let mut host = HostConfig::default().with_sled_config(sc).build()?;
    host.start()?;
    println!("Host should be running in the background");
    Ok(host)
}

macro_rules! integrate_host_and_single_node {
    // macth like arm for macro
    ($a:ty) => {
        // macro expand to this code

        {
            type N = $a;
            println!("Running test on: {}", std::any::type_name::<N>());

            let _host = start_host().unwrap();

            // Get the host up and running
            let node: Node<Blocking, N, Idle, Pose> = NodeConfig::new("pose").build().unwrap();
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
    };
}

#[test]
fn integrate_host_and_single_node_macros() {
    integrate_host_and_single_node!(Tcp);
    #[cfg(not(feature = "quic"))]
    integrate_host_and_single_node!(Udp);
    #[cfg(feature = "quic")]
    integrate_host_and_single_node!(Quic);
}

macro_rules! custom_msg {
    // macth like arm for macro
    ($a:ty) => {
        // macro expand to this code

        {
            type N = $a;
            println!("Running test on: {}", std::any::type_name::<N>());

            let _host = start_host().unwrap();

            // Get the host up and running
            let node: Node<Blocking, N, Idle, Pose> = NodeConfig::new("pose").build().unwrap();
            let node = node.activate().unwrap();

            for i in 0..5 {
                // Could get this by reading a GPS, for example
                let pose = Pose {
                    x: i as f32,
                    y: i as f32,
                };

                let mut msg: Msg<Pose> = Msg::new(MsgType::Set, "pose", pose.clone());
                msg.set_timestamp(Utc::now());

                node.publish_msg(msg).unwrap();
                thread::sleep(Duration::from_millis(10));
                let result = node.request().unwrap();
                println!("Got position: {:?}", result);

                assert_eq!(pose, result.data);
            }
        }
    };
}

#[test]
fn custom_msg_macros() {
    custom_msg!(Tcp);
    #[cfg(not(feature = "quic"))]
    custom_msg!(Udp);
    #[cfg(feature = "quic")]
    custom_msg!(Quic);
}

macro_rules! request_non_existent_topic {
    // macth like arm for macro
    ($a:ty) => {
        // macro expand to this code

        {
            type N = $a;
            println!("Running test on: {}", std::any::type_name::<N>());
            // Test code goes below here

            let _host = start_host().unwrap();

            let node: Node<Blocking, N, Idle, Pose> =
                NodeConfig::new("doesnt_exist").build().unwrap();
            let node = node.activate().unwrap();

            // Requesting a topic that doesn't exist should return a recoverable error
            for i in 0..5 {
                println!("on loop: {}", i);
                let result = node.request();
                dbg!(&result);
                thread::sleep(Duration::from_millis(50));
            }
        }
    };
}

#[test]
fn request_non_existent_topic_macros() {
    request_non_existent_topic!(Tcp);
    #[cfg(not(feature = "quic"))]
    request_non_existent_topic!(Udp);
    #[cfg(feature = "quic")]
    request_non_existent_topic!(Quic);
}

macro_rules! node_send_options {
    // macth like arm for macro
    ($a:ty) => {
        // macro expand to this code

        {
            type N = $a;
            println!("Running test on: {}", std::any::type_name::<N>());
            // Test code goes below here

            let _host = start_host().unwrap();

            let node: Node<Blocking, N, Idle, Pose> =
                NodeConfig::new("doesnt_exist").build().unwrap();
            let node = node.activate().unwrap();

            // Get the host up and running
            let node_a = NodeConfig::<Blocking, N, Option<f32>>::new("pose")
                .build()
                .unwrap()
                .activate()
                .unwrap();
            let node_b = NodeConfig::<Blocking, N, Option<f32>>::new("pose")
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
    };
}

#[test]
fn node_send_options_macros() {
    node_send_options!(Tcp);
    #[cfg(not(feature = "quic"))]
    node_send_options!(Udp);
    #[cfg(feature = "quic")]
    node_send_options!(Quic);
}

macro_rules! subscription_usize {
    // macth like arm for macro
    ($a:ty) => {
        // macro expand to this code

        {
            type N = $a;
            println!("Running test on: {}", std::any::type_name::<N>());
            // Test code goes below here

            let _host = start_host().unwrap();

            // Get the host up and running
            let writer = NodeConfig::<Blocking, N, usize>::new("subscription")
                .build()
                .unwrap()
                .activate()
                .unwrap();

            // Create a subscription node with a query rate of 100 Hz
            let reader = writer
                .config()
                .clone()
                .build()
                .unwrap()
                .subscribe(Duration::from_millis(10))
                .unwrap();

            for i in 0..5 {
                let test_value = i as usize;
                writer.publish(test_value).unwrap();
                std::thread::sleep(std::time::Duration::from_millis(100));
                assert_eq!(reader.get_subscribed_data().unwrap().data, test_value);
            }
        }
    };
}

#[test]
fn subscription_usize_macros() {
    subscription_usize!(Tcp);
    #[cfg(not(feature = "quic"))]
    subscription_usize!(Udp);
    #[cfg(feature = "quic")]
    subscription_usize!(Quic);
}

macro_rules! no_subscribed_value {
    // macth like arm for macro
    ($a:ty) => {
        // macro expand to this code

        {
            type N = $a;
            println!("Running test on: {}", std::any::type_name::<N>());
            // Test code goes below here

            let _host = start_host().unwrap();

            // Create a subscription node with a query rate of 10 Hz
            let reader = NodeConfig::<Blocking, N, usize>::new("subscription")
                .build()
                .unwrap()
                .subscribe(Duration::from_millis(100))
                .unwrap();

            // Unwrapping on an error should lead to panic
            let _result: usize = reader.get_subscribed_data().unwrap().data;
        }
    };
}

#[should_panic]
#[test]
fn no_subscribed_value_macros() {
    no_subscribed_value!(Tcp);
    #[cfg(not(feature = "quic"))]
    no_subscribed_value!(Udp);
    #[cfg(feature = "quic")]
    no_subscribed_value!(Quic);
}

macro_rules! topics_list {
    // macth like arm for macro
    ($a:ty) => {
        // macro expand to this code

        {
            type N = $a;
            println!("Running test on: {}", std::any::type_name::<N>());
            let host = start_host().unwrap();

            // Get the host up and running
            let topics: Vec<String> = ["a", "b", "c", "d", "e", "f"]
                .iter()
                .map(|x| x.to_string())
                .collect();
            dbg!(&topics);
            let mut nodes = Vec::with_capacity(topics.len());
            for topic in topics.clone() {
                let node: Node<Blocking, N, Idle, usize> = NodeConfig::new(topic).build().unwrap();
                let node = node.activate().unwrap();
                nodes.push(node);
            }

            for i in 0..topics.len() {
                nodes[i].publish(i).unwrap();
                thread::sleep(Duration::from_micros(1_000));
                assert_eq!(host.topics(), nodes[i].topics().unwrap().data);
                let t = if i == 0 {
                    vec![topics[i].to_string()]
                } else {
                    let mut t = topics[0..i + 1]
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>();
                    t.sort();
                    t
                };
                let mut nt = nodes[i].topics().unwrap().data;
                nt.sort();
                assert_eq!(t, nt);
            }
        }
    };
}

#[test]
fn topics_list_macros() {
    topics_list!(Tcp);
    #[cfg(not(feature = "quic"))]
    topics_list!(Udp);
    #[cfg(feature = "quic")]
    topics_list!(Quic);
}

macro_rules! back_nth_operation {
    // macth like arm for macro
    ($a:ty) => {
        // macro expand to this code

        {
            type N = $a;
            println!("Running test on: {}", std::any::type_name::<N>());
            // Test code goes below here
            let _host = start_host().unwrap();

            // Get the host up and running
            let node: Node<Blocking, N, Idle, Pose> = NodeConfig::new("pose").build().unwrap();
            let node = node.activate().unwrap();

            let n = 5;
            for i in 0..n {
                let pose = Pose {
                    x: i as f32,
                    y: i as f32,
                };
                node.publish(pose.clone()).unwrap();
                println!("Published {:?}", &pose);
                assert_eq!(node.request().unwrap().data, pose);
                assert_eq!(node.request_nth_back(0).unwrap().data, pose);
            }
            let back = 3;
            let pose = Pose {
                x: (n - back) as f32,
                y: (n - back) as f32,
            };
            // We use "back + 1" because we're zero-indexed
            let b = node.request_nth_back(back - 1).unwrap().data;
            assert_eq!(node.request_nth_back(back - 1).unwrap().data, pose);
        }
    };
}

#[test]
fn back_nth_operation_macros() {
    back_nth_operation!(Tcp);
    #[cfg(not(feature = "quic"))]
    back_nth_operation!(Udp);
    #[cfg(feature = "quic")]
    back_nth_operation!(Quic);
}

macro_rules! back_nth_operation_fallible {
    // macth like arm for macro
    ($a:ty) => {
        // macro expand to this code

        {
            type N = $a;
            println!("Running test on: {}", std::any::type_name::<N>());
            // Test code goes below here

            let _host = start_host().unwrap();

            // Create a subscription node with a query rate of 10 Hz
            let reader = NodeConfig::<Blocking, N, usize>::new("subscription")
                .build()
                .unwrap()
                .subscribe(Duration::from_millis(100))
                .unwrap();

            // Unwrapping on an error should lead to panic
            let _result: usize = reader.get_subscribed_data().unwrap().data;
        }
    };
}

#[should_panic]
#[test]
fn back_nth_operation_fallible_macros() {
    back_nth_operation_fallible!(Tcp);
    #[cfg(not(feature = "quic"))]
    back_nth_operation_fallible!(Udp);
    #[cfg(feature = "quic")]
    back_nth_operation_fallible!(Quic);
}
