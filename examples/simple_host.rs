use rhiza::host::{Host, HostConfig};
use std::thread;
use std::time::Duration;

fn main() {
    let mut host: Host = HostConfig::new("lo")
        .socket_num(25_000)
        .store_filename("store")
        .build()
        .unwrap();
    host.start().unwrap();

    // Other tasks can operate while the host is running in the background
    thread::sleep(Duration::from_secs(10));

    host.stop().unwrap();
    println!("Host is stopped and maybe dropped?");
    thread::sleep(Duration::from_secs(10));

}
