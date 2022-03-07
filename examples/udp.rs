use bissel::*;
use std::error::Error;

use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let sled_cfg = SledConfig::default().path("store").temporary(true);
    let tcp_cfg = TcpConfig::default("wlp3s0");
    let udp_cfg = UdpConfig::default("lo");
    let mut host = HostConfig::default()
        .with_sled_config(sled_cfg)
        .with_tcp_config(tcp_cfg)
        .with_udp_config(udp_cfg)
        .build()?;
    host.start()?;

    thread::sleep(Duration::from_millis(10_000));
    Ok(())
}
