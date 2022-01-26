use rhiza::*;
use std::thread;
use std::time::Duration;

fn main() {
    // Set up logging
    let file_appender = tracing_appender::rolling::minutely("logs/", "example");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let mut host: Host = HostConfig::new("lo")
        .socket_num(25_000)
        .store_filename("store")
        .build()
        .unwrap();
    host.start().unwrap();
    println!("Host should be running in the background");

    // Get the host up and running
    let addr = "127.0.0.1:25000".parse::<std::net::SocketAddr>().unwrap();
    let mut node: Node<Pose> = NodeConfig::new("pose").host_addr(addr).build().unwrap();
    node.connect().unwrap();

    let mut result = Pose::default();

    for i in 0..5 {
        // Could get this by reading a GPS, for example
        let pose = Pose {
            x: i as f32,
            y: i as f32,
        };

        node.publish_to("pose", pose.clone()).unwrap();
        thread::sleep(Duration::from_millis(1_000));
        result = node.request("pose").unwrap();
        println!("Got position: {:?}", result);

        assert_eq!(pose, result);
    }

    host.stop().unwrap();
}
