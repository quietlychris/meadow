[![crates.io](https://img.shields.io/crates/v/meadow.svg)](https://crates.io/crates/meadow) [![Documentation](https://docs.rs/meadow/badge.svg)](https://docs.rs/meadow) ![CI](https://github.com/quietlychris/meadow/actions/workflows/rust.yml/badge.svg)
# Meadow

`meadow` is an experimental robotics-focused middleware for embedded Linux. It is built with a high preference for catching errors at compile-time over runtime and a focus on developer ergonomics, and can natively operate on any [`serde`](https://serde.rs/)-compatible data type. 

```rust
use meadow::*;

// `meadow` should be able to operate on any `serde`-compatible data types
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

    // Build a Node. Nodes can run over TCP, UDP, or QUIC
    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let node: Node<Udp, Idle, Coordinate> = NodeConfig::new("position")
        .with_config(node::NetworkConfig::<Udp>::default().set_host_addr(addr))
        .build()?;
    // Nodes use strict typestates; without using the activate() method first,
    // the compiler won't let allow publish() or request() methods on an Idle Node
    let node: Node<Udp, Active, Coordinate> = node.activate()?;

    // Since Nodes are statically-typed, the following lines would fail at
    // compile-time due to type errors
    // node.publish(1usize).unwrap()
    // let result: bool = node.request().unwrap();

    node.publish(Coordinate { x: 0.0, y: 0.0 })?;

    // Nodes can also be subscribers, which will request topic updates from the Host
    // at a given rate
    let subscriber = NodeConfig::<Tcp, Coordinate>::new("position")
        .with_config(node::NetworkConfig::<Tcp>::default().set_host_addr(addr))
        .build()?
        .subscribe(std::time::Duration::from_micros(100))?;

    for i in 0..5 {
        // Could get this by reading a GPS, for example
        let c = Coordinate {
            x: i as f32,
            y: i as f32,
        };
        node.publish(c)?;
        let result: Msg<Coordinate> = node.request()?;
        // or could use the value held by the subscribed node
        let subscription = subscriber.get_subscribed_data();
        println!("request: {:?}, subscription: {:?}", result, subscription);
    }

    Ok(())
}
```

## Messaging Patterns 

Meadow is more similar to [ZeroMQ](https://zguide.zeromq.org/docs/chapter1/) than to higher-level frameworks like [ROS/2](https://design.ros2.org/articles/discovery_and_negotiation.html), but uses central coordination process similar to [MOOS-IvP](https://oceanai.mit.edu/ivpman/pmwiki/pmwiki.php?n=Helm.HelmDesignIntro#section2.4), resulting in a star-shaped network topology. 

meadow currently supports the following messaging patterns:

| Protocol | Publish   | Request    | Subscribe | Encryption |
|----------|-----------|------------|-----------|------------|
| TCP      | **X**     | **X**      | **X**     |            |
| UDP      | **X**     | **X**      | **X**     |            |
| QUIC     | **X**     | **X**      | **X**     | **X**      |

Meadow's subscriber functionality currently works a bit differently than many other middlewares; rather than having the most recent data on the subscribed topic pushed to it by the Host upon receive, the Host will the most recent data subscribed topic as a requested rate to the Node, which will cache it locally to be available on-demand rather than on-request.

## Key Dependencies
Under the hood, `meadow` relies on:
* [`sled`](https://github.com/spacejam/sled): High-performance embedded, thread-safe database 
* [`tokio`](https://tokio.rs): Asynchronous runtime, enabling a large number of simultaneous connections
* [`postcard`](https://github.com/jamesmunns/postcard): Efficient `#![no_std]`-compatible, [serde](https://serde.rs/)-based de/serializer designed for embedded or constrained environments. `meadow` should be able to operate native on any `serde`-compatible data types.  

## Benchmarks
Preliminary benchmark data is showing round-trip message times (publish-request-reply) on `locahost` using the `--release` compilation profile, on the README's `Coordinate` data (strongly-typed, 8 bytes) to be <100 microseconds. Statistical benchmarks on different data profiles can be run via [`criterion`](https://github.com/bheisler/criterion.rs) via `cargo bench`.

If you are doing robotics development, `meadow` is probably fast enough to move your data around (unless you're trying to do something like video streaming, in which case you should probably be using dedicated endpoints). 

## Stability
As mentioned above, this library should be considered *experimental*. While the goal is eventually to make this available at a level of maturity, stability, and reliability of other middlewares, `meadow` is not there yet. This library is being used as a dependency for robotics research, with interprocess communication focused on dozens of nodes on `localhost` or a few over a WLAN connection. While `meadow` can work for other use-cases, it has not been extensively tested in those areas. If you are using this library in other areas and come across issues or unexpected behavior, well-formatted bug reports or pull requests addressing those problems are welcomed. 

## Additional Resources
The following projects are built with Meadow:
- [Tutlesim](https://github.com/quietlychris/turtlesim): Simple 2D autonomy simulator
- [Orientation](https://github.com/quietlychris/orientation): Real-time 3D orientation visualization of a BNO055 IMU using Meadow and Bevy

## License

This library is licensed under the [Mozilla Public License, version 2.0](https://www.mozilla.org/en-US/MPL/2.0/FAQ/) (MPL-2.0)
