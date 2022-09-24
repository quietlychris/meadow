use meadow::*;
use tracing::*;

use std::thread;
use std::time::Duration;

fn main() -> Result<(), meadow::Error> {
    let mut host: Host = HostConfig::default().build()?;
    host.start()?;
    println!("Host should be running in the background");

    // Get the host up and running
    let node: Node<Idle, String> = NodeConfig::new("TEAPOT").topic("pose").build().unwrap();
    let node = node.activate()?;

    for i in 0..5 {
        node.publish("Hello".to_string())?;
        println!("published {}", i);
        thread::sleep(Duration::from_millis(250));
        let result = node.request().unwrap();
        println!("Received reply: {}", result);
    }

    println!(
        "The size of an a meadow Host before shutdown is: {}",
        std::mem::size_of_val(&host)
    );
    host.stop()?;
    Ok(())
}
