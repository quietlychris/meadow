use bissel::host::*;
use bissel::node::{Node, NodeConfig};
use bissel::Pose;

use std::error::Error;
use std::thread;
use std::time::Duration;

const LABELS: usize = 36;
fn main() -> Result<(), Box<dyn Error>> {
    // Set up logging
    let file_appender = tracing_appender::rolling::hourly("logs/", "multiple_nodes");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let mut host: Host = HostConfig::new("lo").build()?;
    host.start()?;

    // Run n publish-subscribe loops in different processes
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    for i in 0..LABELS {
        let handle = thread::spawn(move || {
            let thread_num = i;
            let pose = Pose {
                x: thread_num as f32,
                y: thread_num as f32,
            };

            // Create a node
            let name = format!("NODE_{}", i);
            let mut node: Node<Pose> = NodeConfig::new(name).topic("pose").build().unwrap();
            match node.connect() {
                Ok(()) => {
                    println!("NODE_{} connected successfully", i);
                }
                Err(_e) => {
                    panic!("NODE_{} did NOT connect successfully", i);
                }
            }

            node.publish(pose).unwrap();
            thread::sleep(Duration::from_millis((100 / (i + 1)) as u64));
            let result: Pose = node.request().unwrap();
            println!("From thread {}, got: {:?}", thread_num, result);

            println!("Thread {} returning!", i);
            // thread::sleep(Duration::from_millis(10));
            // std::process::exit(0);
        });
        handles.push(handle);
        thread::sleep(Duration::from_millis(1));
    }

    // thread::sleep(Duration::from_secs(10));

    for handle in handles {
        handle.join().unwrap();
    }
    println!("All threads have joined!");
    host.stop()?;
    Ok(())
}
