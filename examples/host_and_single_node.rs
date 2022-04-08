use bissel::*;
use tracing::*;

use std::thread;
use std::time::Duration;

fn main() -> Result<(), bissel::Error> {
    // Set up logging
    let file_appender = tracing_appender::rolling::hourly("logs/", "example");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let mut host: Host = HostConfig::default().build()?;
    host.start()?;
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Idle, Pose> = NodeConfig::new("TEAPOT").topic("pose").build().unwrap();
    let node = node.activate()?;
    info!("Node should now be connected");
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
        println!("published {}", i);
        thread::sleep(Duration::from_millis(250));
        let result = node.request().unwrap();
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
