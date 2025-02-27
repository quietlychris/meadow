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

#[tracing::instrument(skip_all)]
fn main() -> Result<(), meadow::Error> {
    logging();

    #[cfg(feature = "quic")]
    {
        use meadow::host::generate_certs;
        use meadow::host::quic::QuicCertGenConfig;

        generate_certs(QuicCertGenConfig::default());
    }

    type N = Tcp;
    // Configure the Host with logging
    let mut host = {
        let date = chrono::Utc::now();
        let stamp = format!(
            "{}_{}_UTC",
            date.date_naive(),
            date.time().format("%H:%M:%S")
        );
        let sled_cfg = SledConfig::default()
            .path(format!("./logs/{}", stamp))
            // If we wanted to keep the logs, we'd make this `false`
            .temporary(true);
        let mut config = HostConfig::default().with_sled_config(sled_cfg);
        #[cfg(feature = "quic")]
        {
            config = config
                .with_udp_config(None)
                .with_quic_config(Some(QuicConfig::default()))
        }
        config.build()?
    };
    host.start()?;
    println!("Host should be running in the background");

    // thread::sleep(Duration::from_secs(60));

    println!("Starting node");
    // Get the host up and running
    let node: Node<Blocking, N, Idle, Pose> = NodeConfig::new("pose").build().unwrap();
    println!("Idle node built");
    let node = node.activate().unwrap();
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
        dbg!(node.topics()?); // .unwrap();
        println!("Got position: {:?}", result.data);

        assert_eq!(pose, result.data);
    }

    println!(
        "The size of an a meadow Host before shutdown is: {}",
        std::mem::size_of_val(&host)
    );
    assert_eq!(host.topics(), node.topics()?.data);

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
        .with_ansi(true)
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
