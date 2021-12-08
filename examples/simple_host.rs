use rhiza::host::{Host, HostConfig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let mut host: Host = HostConfig::new("lo")
        .socket_num(25_000)
        .store_filename("store")
        .build()
        .unwrap();
    host.start().await.unwrap();

    // Other tasks can operate while the host is running in the background
    sleep(Duration::from_secs(10)).await;

    host.stop().await.unwrap();
}
