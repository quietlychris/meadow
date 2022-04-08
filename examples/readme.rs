use bissel::*;
use serde::{Deserialize, Serialize};

// Any type implementing Debug and serde's De/Serialize traits are Bissel-compatible
// (the standard library Debug and Clone traits are also required)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Coordinate {
    x: f32,
    y: f32,
}

fn main() -> Result<(), bissel::Error> {
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
    // Bissel Nodes use strict typestates; without using the activate() method first,
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
