use rhiza::host::{Host, HostConfig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let cfg = HostConfig::new("wlp3s0")
        .socket_num(25_000)
        .store_filename("store");
    let mut host = Host::from_config(cfg).unwrap();
    host.start().await.unwrap();

    println!("Host should be running in the background");
    // Other tasks can operate while the host is running on it's own thread
    sleep(Duration::from_secs(10)).await;

    // Need to kill host manually
    host.stop().await.unwrap();
}
