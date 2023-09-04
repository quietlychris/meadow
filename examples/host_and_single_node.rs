use meadow::*;
use std::{fs::File, sync::Arc};
use tracing::*;
use tracing_subscriber::{filter, prelude::*};

use std::thread;
use std::time::Duration;

/// Example test struct for docs and tests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Pose {
    pub x: f32,
    pub y: f32,
}

fn main() -> Result<(), meadow::Error> {
    logging();

    // Configure the Host a
    let mut host = {
        let date = chrono::Utc::now();
        let stamp = format!(
            "{}_{}_UTC",
            date.date_naive().to_string(),
            date.time().format("%H:%M:%S").to_string()
        );
        let sled_cfg = SledConfig::default()
            .path(format!("./logs/{}", stamp))
            // If we wanted to keep the logs, we'd make this `false`
            .temporary(true);
        HostConfig::default().with_sled_config(sled_cfg).build()?
    };
    host.start()?;
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Tcp, Idle, Pose> = NodeConfig::new("pose").build().unwrap();
    let node = node.activate()?;
    debug!("Node should now be connected");
    println!(
        "The size of an active meadow Node is: {}",
        std::mem::size_of_val(&node)
    );

    // This following two functions should fail to compile
    // node.publish(NotPose::default())?;
    // let not_pose: NotPose = node.request()?;

    for i in 0..5 {
        // Could get this by reading a GPS, for example
        let pose = Pose {
            x: i as f32,
            y: i as f32,
        };

        node.publish(pose.clone())?;
        println!("published {}", i);
        thread::sleep(Duration::from_millis(250));
        let result: Msg<Pose> = node.request().unwrap();
        println!("Got position: {:?}", result);

        assert_eq!(pose, result.data);
    }

    println!(
        "The size of an a meadow Host before shutdown is: {}",
        std::mem::size_of_val(&host)
    );
    let topics = host.topics();
    for topic in &topics {
        if let Some(ref db) = host.store {
            let db = db.clone();
            let tree = db.open_tree(topic.as_bytes()).unwrap();
            println!("Topic {} has {} stored values", topic, tree.len());
        }
    }

    Ok(())
}

fn logging() {
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
                .with_filter(filter::LevelFilter::INFO),
        )
        .init();
}
