[![crates.io](https://img.shields.io/crates/v/meadow.svg)](https://crates.io/crates/meadow) [![Documentation](https://docs.rs/meadow/badge.svg)](https://docs.rs/meadow) ![CI](https://github.com/quietlychris/meadow/actions/workflows/rust.yml/badge.svg)
# Meadow

`meadow` is an experimental robotics-focused middleware for embedded Linux. It is built with a high preference for catching errors at compile-time over runtime and a focus on developer ergonomics. 

```rust
use meadow::*;
use serde::{Deserialize, Serialize};

// Any type implementing Debug and serde's De/Serialize traits are meadow-compatible
// (the standard library Debug and Clone traits are also required)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Coordinate {
    x: f32,
    y: f32,
}

fn main() -> Result<(), meadow::Error> {
    // The Host is running on localhost, but any network interface such as WiFi
    // or Ethernet are available as well
    let mut host: Host = HostConfig::default().build()?;
    host.start()?;
    // Other tasks can operate while the host is running in the background

    // Build a Node
    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let node: Node<Idle, Coordinate> = NodeConfig::new("GPS_NODE")
        .topic("position")
        .with_tcp_config(node::TcpConfig::default().set_host_addr(addr))
        .build()?;
    // Meadow Nodes use strict typestates; without using the activate() method first,
    // the compiler won't let allow publish() or request() methods on an Idle Node
    let node: Node<Active, Coordinate> = node.activate()?;

    // Since Nodes are statically-typed, the following lines would fail at
    // compile-time due to type errors
    // node.publish(1usize).unwrap()
    // let result: bool = node.request().unwrap();

    node.publish(Coordinate { x: 0.0, y: 0.0 })?;

    // Nodes can also be subscribers, which will request topic updates from the Host
    // at a given rate
    let subscriber = NodeConfig::<Coordinate>::new("GPS_SUBSCRIBER")
        .topic("position")
        .with_tcp_config(node::TcpConfig::default().set_host_addr(addr))
        .build()?
        .subscribe(std::time::Duration::from_micros(100))?;

    for i in 0..5 {
        // Could get this by reading a GPS, for example
        let c = Coordinate {
            x: i as f32,
            y: i as f32,
        };
        node.publish(c)?;
        let result = node.request()?;
        // or could use the value held by the subscribed node
        let subscription = subscriber.get_subscribed_data();
        println!("request: {:?}, subscription: {:?}", result, subscription);
    }

    host.stop()?;
    Ok(())
}
```

## Messaging Patterns 

Meadow is more similar to [ZeroMQ](https://zguide.zeromq.org/docs/chapter1/) than to higher-level frameworks like [ROS/2](https://design.ros2.org/articles/discovery_and_negotiation.html), but uses central coordination process similar to [MOOS-IvP](https://oceanai.mit.edu/ivpman/pmwiki/pmwiki.php?n=Helm.HelmDesignIntro#section2.4), resulting in a star-shaped network topology. 

meadow currently supports the following messaging patterns:

| Protocol | Publish   | Request    | Subscribe |
|----------|-----------|------------|-----------|
| TCP      | **X**     | **X**      | **X**     |
| UDP      | **X**     |            |           |


## Key Dependencies
Under the hood, `meadow` relies on:
* [`sled`](https://github.com/spacejam/sled): High-performance embedded, thread-safe database 
* [`tokio`](https://tokio.rs): Asynchronous runtime, enabling a large number of simultaneous connections
* [`postcard`](https://github.com/jamesmunns/postcard): Efficient `#![no_std]`-compatible, [serde](https://serde.rs/)-based de/serializer designed for embedded or constrained environments 

## Benchmarks
Preliminary benchmark data is showing round-trip message times (publish-request-reply) on `locahost` using the `--release`
compilation profile, on the README's `Coordinate` data (strongly-typed, 8 bytes) to be ~100 microseconds.

Additional benchmarking information can be found using `cargo run --release --example benchmark`. 

## Additional Resources
The following projects are built with Meadow:
- [Tutlesim](https://github.com/quietlychris/turtlesim): Simple 2D autonomy simulator
- [Orientation](https://github.com/quietlychris/orientation): Real-time 3D orientation visualization of a BNO055 IMU using Meadow and Bevy

## License

This library is licensed under the Mozilla Public License, version 2.0 (MPL-2.0)
