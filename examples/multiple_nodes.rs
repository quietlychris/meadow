use rhiza::host::*;
use rhiza::node::{Node, NodeConfig};
use rhiza::Pose;

use std::thread;
use std::time::Duration;

// const LABELS: [&str; 6] = ["ALL", "ALONG", "WATCHTOWER", "PRINCES", "KEPT", "VIEW"];
const LABELS: usize = 100;
fn main() {
    // Set up logging
    let file_appender = tracing_appender::rolling::hourly("logs/", "multiple_nodes");
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
    for i in 0..LABELS {
        let handle = thread::spawn(move || {
            let thread_num = i;
            let pose = Pose {
                x: thread_num as f32,
                y: thread_num as f32,
            };

            // Create our node
            let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
            let name = format!("LABEL_{}", i);
            let mut node: Node<Pose> = NodeConfig::new(name).host_addr(addr).build().unwrap();
            match node.connect() {
                Ok(()) => {
                    println!("NODE_{} connected successfully", i);
                }
                Err(e) => {
                    panic!("NODE_{} did NOT connect successfully", i);
                }
            }

            for i in 0..20 {
                node.publish_to("pose", pose.clone()).unwrap();
                thread::sleep(Duration::from_millis(i * 5 as u64));
                let result: Pose = node.request("pose").unwrap();
                // println!("From thread {}, got: {:?}", thread_num, result);
            }
        });
        handles.push(handle);
        thread::sleep(Duration::from_millis(1));
    }
    println!("I'm a little teapot!");

    thread::sleep(Duration::from_secs(10));

    for handle in handles {
        handle.join().unwrap();
    }
    host.stop().unwrap();
}
