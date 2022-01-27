use bissel::*;

use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    // Set up logging
    let file_appender = tracing_appender::rolling::minutely("logs/", "example");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let mut host: Host = HostConfig::new("lo").build()?;
    host.start()?;
    println!("Host should be running in the background");

    // Get the host up and running
    let mut node: Node<Pose> = NodeConfig::new("TEAPOT").topic("pose").build().unwrap();
    node.connect()?;
    println!(
        "The size of an active bissel Node is: {}",
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
        thread::sleep(Duration::from_millis(1_000));
        let result = node.request()?;
        println!("Got position: {:?}", result);

        assert_eq!(pose, result);
    }

    println!(
        "The size of an a bissel Host before shutdown is: {}",
        std::mem::size_of_val(&host)
    );
    host.stop()?;
    Ok(())
}