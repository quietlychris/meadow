# bissel

`bissel` is an early-stage robotics-focused publish/request middleware for embedded Linux. It uses a star-shaped network topology, with a focus on ease-of-use and transparent design and operation. It is more similar to [ZeroMQ](https://zguide.zeromq.org/docs/chapter1/) than to higher-level frameworks like [ROS/2](https://design.ros2.org/articles/discovery_and_negotiation.html), but uses central coordination process similar to [MOOS-IvP](https://oceanai.mit.edu/ivpman/pmwiki/pmwiki.php?n=Helm.HelmDesignIntro#section2.4). 

Under the hood, `bissel` relies on:
* [`sled`](https://github.com/spacejam/sled): High-performance embedded, thread-safe database 
* [`tokio`](https://tokio.rs): Asynchronous runtime, enabling a large number of simultaneous connections
* [`postcard`](https://github.com/jamesmunns/postcard): Efficient `#![no_std]`-compatible, [serde](https://serde.rs/)-based de/serializer designed for embedded or constrained environments 

### Host 
```rust
// A simple host, which can be run remotely or co-located
// with the attached nodes 
use bissel::host::{Host, HostConfig};
use std::thread;
use std::time::Duration;

fn main() {
    let mut host: Host = HostConfig::new("lo")
        .socket_num(25_000)
        .store_filename("store")
        .build()
        .unwrap();
    host.start().unwrap();

    // Other tasks can operate while the host is running in the background
    thread::sleep(Duration::from_secs(5));

    host.stop().unwrap();
}

```

### Node
```rust,no_run
// A simple node (client-side)
use bissel::node::{Node, NodeConfig};
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Coordinate {
    x: f32,
    y: f32,
}

fn main() {

    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let mut node: Node<Coordinate> = NodeConfig::new("GPS_NODE")
        .topic("position")
        .host_addr(addr)
        .build()
        .unwrap();
    node.connect().unwrap();

    let c = Coordinate { x: 4.0, y: 4.0 };
    node.publish(c).unwrap();

    // The following lines would fail at compile-time due to type errors!
    // node.publish(1usize).unwrap()
    // let result: bool = node.request().unwrap();

    for i in 0..5 {
        // Could get this by reading a GPS, for example
        let c = Coordinate { x: i as f32, y: i as f32 };
        node.publish(c).unwrap();
        thread::sleep(Duration::from_millis(1_000));
        let result = node.request().unwrap();
        println!("Got position: {:?}", result);
    }
}
```
## License

This library is currently licensed under the Lesser GNU Public License, version 3.0 (LGPL-3.0).
