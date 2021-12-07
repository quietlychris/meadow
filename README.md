# rhiza

This is an early-stage robotics-focused publish/request middleware for embedded Linux. It uses a star-shaped network topology, with a focus on ease-of-use and transparent design and operation. It is more similar to [ZeroMQ](https://zguide.zeromq.org/docs/chapter1/) than to higher-level frameworks like [ROS/2](https://design.ros2.org/articles/discovery_and_negotiation.html), but uses central coordination process similar to [MOOS-IvP](https://oceanai.mit.edu/ivpman/pmwiki/pmwiki.php?n=Helm.HelmDesignIntro#section2.4). 

Under the hood, `rhiza` relies on:
* [`sled`](https://github.com/spacejam/sled): High-performance embedded, thread-safe database 
* [`tokio`](https://tokio.rs): Asynchronous runtime, enabling a large number of simultaneous connections
* [`postcard`](https://github.com/jamesmunns/postcard): Efficient `#![no_std]`-compatible, [serde](https://serde.rs/)-based de/serializer designed for embedded or constrained environments 

### Host 
```rust
// A simple host, which can be run remotely or co-located
// with the attached nodes 
use rhiza::host::{Host, HostConfig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // By default, Rhiza Hosts run on the localhost, but other interfaces
    // are allowed, allowing connections over Ethernet or WiFi
    let cfg = HostConfig::new("lo")  
        .socket_num(25_000)       // Port 25000 is the default address  
        .store_filename("store"); // sled databases allow persistence across reboots
    let mut host = Host::from_config(cfg).unwrap();
    host.start().await.unwrap();

    // Other tasks can operate while the host is running in the background
    sleep(Duration::from_secs(10)).await;

    host.stop().await.unwrap();
}
```

### Node
```rust
// A simple node (client-side)
use rhiza::node::{Node, NodeConfig};
use tokio::time::{sleep, Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Coordinate {
    x: f32,
    y: f32,
}

#[tokio::main]
async fn main() {
    // This is the default TCP address of the central Rhiza Host
    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let cfg: NodeConfig<Coordinate> = NodeConfig::new("pose").host_addr(addr);
    let mut node: Node<Coordinate> = Node::from_config(cfg);
    // Each node establishs a TCP connection with central host
    node.connect().await.unwrap();

    let c = Coordinate { x: 4.0, y: 4.0 };
    node.publish_to("pose", c).await.unwrap();

    for _ in 0..5 {
        // Could get this by reading a GPS, for example
        let c = Coordinate { x: 4.0, y: 4.0 };
        node.publish_to("pose", c).await.unwrap();
        sleep(Duration::from_millis(1_000)).await;
        let result: Coordinate = node.request("pose").await.unwrap();
        println!("Got position: {:?}", result);
    }
}
```
