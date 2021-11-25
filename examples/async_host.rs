use rhiza::host::{Host, HostConfig};

use std::error::Error;
use std::sync::Arc;
use std::thread;

use tokio::signal;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn run_host() {
    let cfg = HostConfig::new("lo")
        .socket_num(25_000)
        .store_name("store");
    let mut host = Host::from_config(cfg).await.unwrap();
    host.start().await.unwrap();
}

fn main() {
    let handle = thread::spawn(|| {
        run_host();
    });

    println!("Host should be running in the background");
    // Do whatever else you'd like while there's a host in the background
    for i in 0..60 {
        thread::sleep(std::time::Duration::from_millis(1_000));
    }
    // Need to kill host w/ Ctrl^C
    handle.join().unwrap();
}
