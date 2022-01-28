use std::error::Error;
use std::net::IpAddr;

pub fn get_ip(interface_name: &str) -> Result<String, Box<dyn Error>> {
    let interface = pnet::datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .unwrap_or_else(|| {
            panic!(
                "IP address for interface {} does n
        ot exist or is not up",
                interface_name
            )
        });

    // dbg!(&interface);
    let source_ip = interface
        .ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .map(|ip| match ip.ip() {
            IpAddr::V4(ip) => ip,
            _ => unreachable!(),
        })
        .unwrap_or_else(|| {
            panic!(
                "IP address for interface {} does n
        ot exist or is not up",
                interface_name
            )
        })
        .to_string();

    Ok(source_ip)
}
