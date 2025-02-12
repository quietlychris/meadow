use meadow::prelude::*;

/// Example test struct for docs and tests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Pose {
    pub x: f32,
    pub y: f32,
}

fn main() -> Result<(), meadow::Error> {
    logging();

    #[cfg(feature = "quic")]
    {
        use meadow::host::generate_certs;
        use meadow::host::quic::QuicCertGenConfig;

        generate_certs(QuicCertGenConfig::default());
    }

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

    let n = 5;
    for i in 0..n {
        let pose = Pose {
            x: i as f32,
            y: i as f32,
        };
        host.insert("pose", pose.clone())?;
        assert_eq!(host.get::<Pose>("pose")?.data, pose);
        assert_eq!(host.get_nth_back::<Pose>("pose", 0)?.data, pose);
    }
    let back = 3;
    let pose = Pose {
        x: (n - back) as f32,
        y: (n - back) as f32,
    };
    // We use "back + 1" because we're zero-indexed
    assert_eq!(host.get_nth_back::<Pose>("pose", back - 1)?.data, pose);
    let result = host.get_nth_back::<Pose>("pose", 10);
    match result {
        Err(Error::NoNthValue) => (),
        _ => panic!("Should not be okay!"),
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
