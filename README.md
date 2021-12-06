# rhiza

This is an early-stage robotics-focused publish/request middleware for embedded Linux and compatibility with  `#![no_std]` embedded systems. It uses a star-shaped network topology, with a focus on ease-of-use and transparent operation.  

Under the hood, this `rhiza` relies on:
* [`sled`](https://github.com/spacejam/sled): An high-performance embedded, thread-safe database 
* [`tokio`](https://tokio.rs): An async runtime, enabling a large number of simultaneous connections
* [`postcard`](https://github.com/jamesmunns/postcard): An efficient `#![no_std]`, [serde](https://serde.rs/)-compatible de/serializer designed for embedded or constrained environments 

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
    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let cfg: NodeConfig<Coordinate> = NodeConfig::new("pose").host_addr(addr);
    let mut node: Node<Coordinate> = Node::from_config(cfg);
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
### Host 
```rust
// A simple host, which can be run remotely or co-located
// with the attached nodes 
use rhiza::host::{Host, HostConfig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let cfg = HostConfig::new("lo") 
        .socket_num(25_000)
        .store_filename("store");
    let mut host = Host::from_config(cfg).unwrap();
    host.start().await.unwrap();

    // Other tasks can operate while the host is running in the background
    sleep(Duration::from_secs(10)).await;

    host.stop().await.unwrap();
}

```