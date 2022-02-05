[![crates.io](https://img.shields.io/crates/v/bissel.svg)](https://crates.io/crates/bissel) [![Documentation](https://docs.rs/bissel/badge.svg)](https://docs.rs/bissel) ![CI](https://github.com/quietlychris/bissel/actions/workflows/rust.yml/badge.svg)
# bissel

`bissel` is an experimental robotics-focused publish/request middleware for embedded Linux. It uses a star-shaped network topology, with a focus on ease-of-use and transparent design and operation. It is more similar to [ZeroMQ](https://zguide.zeromq.org/docs/chapter1/) than to higher-level frameworks like [ROS/2](https://design.ros2.org/articles/discovery_and_negotiation.html), but uses central coordination process similar to [MOOS-IvP](https://oceanai.mit.edu/ivpman/pmwiki/pmwiki.php?n=Helm.HelmDesignIntro#section2.4). 

Under the hood, `bissel` relies on:
* [`sled`](https://github.com/spacejam/sled): High-performance embedded, thread-safe database 
* [`tokio`](https://tokio.rs): Asynchronous runtime, enabling a large number of simultaneous connections
* [`postcard`](https://github.com/jamesmunns/postcard): Efficient `#![no_std]`-compatible, [serde](https://serde.rs/)-based de/serializer designed for embedded or constrained environments 

```rust
use bissel::*;
use serde::{Deserialize, Serialize};

// Any type implementing Debug and serde's De/Serialize traits are Bissel-compatible
// (the standard library Debug and Clone traits are also required)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Coordinate {
    x: f32,
    y: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The Host is running on localhost, but any network interface such as WiFi
    // or Ethernet are available as well
    let mut host: Host = HostConfig::new("lo")
        .socket_num(25_000)
        .store_filename("store")
        .build()
        .unwrap();
    host.start().unwrap();
    // Other tasks can operate while the host is running in the background
    
    // Build a Node
    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let node: Node<Idle, Coordinate> = NodeConfig::new("GPS_NODE")
        .topic("position")
        .host_addr(addr)
        .build()?;
    // Bissel Nodes use strict typestates; without using the connect() method first,
    // the compiler won't let you use the publish() or request() methods on an Idle Node
    let mut node: Node<Active, Coordinate> = node.connect()?;

    // Nodes can also be subscribers, which will request topic updates from the Host
    // at a given rate
    let subscriber: Node<Subscription, Coordinate> = NodeConfig::new("GPS_SUBSCRIBER")
        .topic("position")
        .host_addr(addr)
        .build()?
        .subscribe(std::time::Duration::from_millis(100))?;

    let c = Coordinate { x: 4.0, y: 4.0 };
    node.publish(c).unwrap();

    // Since Nodes are statically-typed, the following lines would fail at 
    // compile-time due to type errors!
    // node.publish(1usize).unwrap()
    // let result: bool = node.request().unwrap();

    for i in 0..5 {
        // Could get this by reading a GPS, for example
        let c = Coordinate { x: i as f32, y: i as f32 };
        node.publish(c)?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        let result = node.request()?;
        // or could use the value held by the subscribed node
        let subscription = subscriber.get_subscribed_data().unwrap().unwrap();
        println!("Got position: {:?}", result);
    }

    // host.stop()?;
    Ok(())
}

```

## License

This library is licensed under the Mozilla Public License, version 2.0 (MPL-2.0)
