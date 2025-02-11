use meadow::prelude::*;
use std::thread;
use std::time::Duration;

use tracing::*;

/// Example test struct for docs and tests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Pose {
    pub x: f32,
    pub y: f32,
}

fn main() -> Result<(), meadow::Error> {
    logging();

    let sc = SledConfig::new().temporary(true);
    let mut host = HostConfig::default().with_sled_config(sc).build().unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let topics: Vec<String> = ["a", "b", "c", "d", "e", "f"]
        .iter()
        .map(|x| x.to_string())
        .collect();
    dbg!(&topics);
    let mut nodes = Vec::with_capacity(topics.len());
    for topic in topics.clone() {
        let node: Node<Blocking, Tcp, Idle, usize> = NodeConfig::new(topic).build().unwrap();
        let node = node.activate().unwrap();
        nodes.push(node);
    }

    for i in 0..topics.len() {
        nodes[i].publish(i).unwrap();
        println!("Published {} on topic {}", i, nodes[i].topic());
        let t = nodes[i].topics().unwrap();
        dbg!(&t);
        assert_eq!(host.topics().unwrap(), nodes[i].topics().unwrap().data);
        let t = if i == 0 {
            vec![topics[i].to_string()]
        } else {
            let mut t = topics[0..i + 1]
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
            t.sort();
            t
        };
        let mut nt = nodes[i].topics().unwrap().data;
        nt.sort();
        assert_eq!(t, nt);
    }
    Ok(())
}

fn logging() {
    use std::{fs::File, sync::Arc};
    use tracing_subscriber::{filter, prelude::*};

    // A layer that logs events to a file.
    let file = File::create("logs/debug.log");
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Error: {:?}", error),
    };

    let log = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(false)
        .with_line_number(true)
        .with_writer(Arc::new(file));

    tracing_subscriber::registry()
        .with(
            log
                // Add an `INFO` filter to the stdout logging layer
                .with_filter(filter::LevelFilter::INFO), // .with_filter(filter::LevelFilter::WARN)
                                                         // .with_filter(filter::LevelFilter::ERROR)
        )
        .init();
}
