use bissel::*;

use clap::{App, Arg};

#[tokio::main]
async fn main() {
    start_logging();

    let matches = App::new("Bissel Host")
        .version("0.3")
        .author("Christopher Moran <cmoran@cmoran.xyz>")
        .about("Start a Bissel host")
        .arg(
            Arg::new("interface")
                .short('i')
                .long("interface")
                .default_value("lo")
                .help("Sets the proper network interface"),
        )
        .arg(
            Arg::new("socket")
                .short('s')
                .long("tcp_socket_num")
                .default_value("25000")
                .help("Sets an alternative TCP socket"),
        )
        .arg(
            Arg::new("store_filename")
                .short('f')
                .long("store_filename")
                .default_value("store")
                .help("Sets the filename for the `sled`-based key-value store"),
        )
        .get_matches();

    let interface: String = matches.value_of("interface").unwrap().to_string();
    let store_filename: String = matches.value_of("store_filename").unwrap().to_string();
    let _socket: usize = matches.value_of("socket").unwrap().parse().unwrap();

    let mut host: Host = HostConfig::default()
        .with_udp_config(Some(host::UdpConfig::default(&interface)))
        .with_tcp_config(Some(host::TcpConfig::default(&interface)))
        .with_sled_config(SledConfig::default().path(store_filename))
        .build()
        .unwrap();
    host.start().unwrap();

    println!("Bissel Host should be running");
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
    // Other tasks can operate while the host is running on it's own thread
    host.stop().unwrap();
}

fn start_logging() {
    let file_appender = tracing_appender::rolling::hourly("logs/", "start_host");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();
}
