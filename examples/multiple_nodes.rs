use rhiza::host::*;
use rhiza::node::{Node, NodeConfig};
use rhiza::Pose;

use std::thread;
use std::time::Duration;

const LABELS: [&str; 6] = ["ALL", "ALONG", "WATCHTOWER", "PRINCES", "KEPT", "VIEW"];
fn main() {
    // Set up logging
    let file_appender = tracing_appender::rolling::minutely("logs/", "multiple_nodes");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let mut host: Host = HostConfig::new("lo")
        .socket_num(25_000)
        .store_filename("store")
        .build()
        .unwrap();
    host.start().unwrap();

    // Run n publish-subscribe loops in different processes
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    for i in 0..LABELS.len() {
        let handle = thread::spawn(move || {
            let thread_num = i;
            let pose = Pose {
                x: thread_num as f32,
                y: thread_num as f32,
            };

            // Create our node
            let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
            let mut node: Node<Pose> = NodeConfig::new(LABELS[i]).host_addr(addr).build().unwrap();
            node.connect().unwrap();

            for i in 0..100 {
                node.publish_to("pose", pose.clone()).unwrap();
                thread::sleep(Duration::from_millis(i * 5 as u64));
                let result: Pose = node.request("pose").unwrap();
                println!("From thread {}, got: {:?}", thread_num, result);
            }
        });
        handles.push(handle);
        thread::sleep(Duration::from_millis(100));
    }

    thread::sleep(Duration::from_secs(10));

    for handle in handles {
        handle.join().unwrap();
    }
    host.stop().unwrap();
}
