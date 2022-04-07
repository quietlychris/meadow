use crate::Error;
use std::net::IpAddr;

/// Get the IP address on a network interface for this computer
pub fn get_ip(interface_name: &str) -> Result<String, Error> {
    let interface = match pnet_datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == interface_name)
    {
        Some(interfaces) => interfaces,
        None => return Err(Error::InvalidInterface),
    };

    let source_ip = match interface
        .ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .map(|ip| match ip.ip() {
            IpAddr::V4(ip) => ip,
            _ => unreachable!(),
        }) {
        Some(name) => name.to_string(),
        None => return Err(Error::InvalidInterface),
    };

    Ok(source_ip)
}
