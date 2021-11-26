use rhiza::host::{Host, HostConfig};
use std::thread;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn run_host() {
    let cfg = HostConfig::new("wlp5s0")
        .socket_num(25_000)
        .store_name("store");
    let mut host = Host::from_config(cfg).await.unwrap();
    host.start().await.unwrap(); // This runs indefinitely
}

fn main() {
    // We could run the host directly in a #[tokio::main] main function
    // or hand it off to a
    let handle = thread::spawn(|| {
        run_host();
    });

    println!("Host should be running in the background");
    // Other tasks can operate while the host is running on it's own thread
    thread::sleep(std::time::Duration::from_secs(15));

    // Need to kill host manually
    handle.join().unwrap();
}
