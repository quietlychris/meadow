mod active;
mod idle;
mod subscription;

use crate::msg::{GenericMsg, Message, Msg};
use std::convert::TryInto;
use tokio::net::UdpSocket;

use tracing::*;

use crate::Error;
use std::net::SocketAddr;

#[inline]
pub async fn await_response<T: Message>(
    socket: &UdpSocket,
    buf: &mut [u8],
) -> Result<Msg<T>, Error> {
    info!("await_response");
    match socket.readable().await {
        Ok(_) => (),
        Err(_e) => return Err(Error::AccessSocket),
    };
    info!("readable!");

    for i in 0..10 {
        match socket.recv(buf).await {
            Ok(0) => {
                info!("await_response received zero bytes");
                continue;
            },
            Ok(n) => {
                info!("await_response received {} bytes",n);
                let bytes = &buf[..n];
                match postcard::from_bytes::<GenericMsg>(bytes) {
                    Ok(generic) => {
                        if let Ok(msg) = TryInto::<Msg<T>>::try_into(generic) {
                            return Ok(msg);
                        } else {
                            return Err(Error::Deserialization);
                        }
                    }
                    Err(_e) => return Err(Error::Deserialization),
                }
            }
            Err(e) => {
                
                if e.kind() == std::io::ErrorKind::WouldBlock {error!("Would block");}
                continue;
            }
        }
    }
    Err(Error::BadResponse)
}

#[inline]
async fn send_msg(
    socket: &UdpSocket,
    packet_as_bytes: Vec<u8>,
    host_addr: SocketAddr,
) -> Result<usize, Error> {
    match socket.writable().await {
        Ok(_) => (),
        Err(_e) => return Err(Error::AccessSocket),
    };

    // Write the request
    for _ in 0..10 {
        match socket.send_to(&packet_as_bytes, host_addr).await {
            Ok(n) => return Ok(n),
            Err(_e) => {}
        }
    }
    Err(Error::BadResponse)
}
