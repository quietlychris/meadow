mod active;
mod idle;
mod subscription;

use crate::msg::{GenericMsg, Message, Msg};
use std::convert::TryInto;
use tokio::net::UdpSocket;

use tracing::*;

use crate::error::Error;
use std::io::{Error as IoError, ErrorKind};
use std::net::SocketAddr;

#[inline]
pub async fn await_response<T: Message>(
    socket: &UdpSocket,
    buf: &mut [u8],
) -> Result<Msg<T>, Error> {
    socket.readable().await?;
    for _ in 0..10 {
        match socket.recv(buf).await {
            Ok(0) => {
                info!("await_response received zero bytes");
                continue;
            }
            Ok(n) => {
                info!("await_response received {} bytes", n);
                let bytes = &buf[..n];
                let generic = postcard::from_bytes::<GenericMsg>(bytes)?;
                info!("Generic: {:?}", &generic);
                let msg: Msg<T> = generic.try_into()?;
                return Ok(msg);
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    error!("Would block");
                }
                continue;
            }
        }
    }
    Err(Error::Io(IoError::new(
        ErrorKind::TimedOut,
        "Didn't receive a response within 10 cycles!",
    )))
}

#[inline]
async fn send_msg(
    socket: &UdpSocket,
    packet_as_bytes: Vec<u8>,
    host_addr: SocketAddr,
) -> Result<usize, Error> {
    socket.writable().await?;
    // NOTE: This used to be done 10 times in a row to make sure it got through
    let n = socket.send_to(&packet_as_bytes, host_addr).await?;
    Ok(n)
}
