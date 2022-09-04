use meadow::*;

use std::thread;
use std::time::Duration;

/// Example test struct for docs and tests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Pose {
    pub x: f32,
    pub y: f32,
}

const LABELS: usize = 36;
fn main() -> Result<(), meadow::Error> {
    // Set up logging
    start_logging();

    let mut host: Host = HostConfig::default().build()?;
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
            let node: Node<Idle, Pose> = NodeConfig::new(name).topic("pose").build().unwrap();
            let node = match node.activate() {
                Ok(node) => {
                    println!("NODE_{} connected successfully", i);
                    node
                }
                Err(_e) => {
                    panic!("NODE_{} did NOT connect successfully", i);
                }
            };

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

fn start_logging() {
    let file_appender = tracing_appender::rolling::hourly("logs/", "multiple_nodes");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();
}
